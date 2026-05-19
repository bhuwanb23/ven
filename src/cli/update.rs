//! `ven update` — self-update the running `ven` binary (and its sibling
//! `ven-launcher`) from the latest GitHub release.
//!
//! Flow:
//!   1. Resolve current install dir from `current_exe().parent()`.
//!   2. Hit GitHub's release API for the requested tag (default: latest).
//!   3. Compare against the compile-time version. If equal and `--force` is
//!      off, exit 0 ("already up to date"). With `--check`, always exit
//!      without downloading.
//!   4. Pick the platform-specific "combined" asset (`ven-{os}-{arch}.zip`
//!      on Windows, `ven-{os}-{arch}.tar.gz` everywhere else).
//!   5. Write-probe the install dir. If it fails, re-launch self elevated
//!      (UAC on Windows, sudo on Unix) with `--reentry` and exit.
//!   6. Download the asset, verify it against the SHA256SUMS asset from the
//!      same release, extract to a temp dir, locate the two binaries.
//!   7. Self-replace each binary in-place. On Windows the running .exe is
//!      first renamed to `*.exe.old` (allowed even while running), then the
//!      new bytes are written to the original path. On Unix we unlink +
//!      write; the open file descriptor in our own process keeps pointing
//!      at the old inode, so we never SIGSEGV ourselves.
//!
//! Asset names mirror `.github/workflows/release.yml` and the manifest
//! consumed by the website (`ven_website/public/releases-manifest.json`).

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::core::integrity::{download_to_file, fetch_manifest_sha256, verify_sha256};

const REPO: &str = "bhuwanb23/ven";
const USER_AGENT: &str = "ven-update";

// -----------------------------------------------------------------------------
// GitHub API types (only the fields we care about)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: Option<String>,
    html_url: Option<String>,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize, Clone)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    #[allow(dead_code)]
    size: Option<u64>,
}

// -----------------------------------------------------------------------------
// JSON report (`ven update --json`)
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct UpdateReport {
    current: String,
    target: String,
    up_to_date: bool,
    action: &'static str,
    install_dir: String,
    install_mode: String,
    asset: String,
    repo: String,
    release_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// -----------------------------------------------------------------------------
// Entry point
// -----------------------------------------------------------------------------

pub fn cmd_update(
    check: bool,
    version: Option<&str>,
    yes: bool,
    force: bool,
    json: bool,
    reentry: bool,
) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // 1. Find the install dir from the running executable.
    let current_exe = std::env::current_exe()
        .context("Could not resolve the path of the running `ven` executable")?;
    let install_dir = current_exe
        .parent()
        .ok_or_else(|| {
            anyhow!(
                "ven executable has no parent dir: {}",
                current_exe.display()
            )
        })?
        .to_path_buf();
    let install_mode = classify_install_dir(&install_dir);

    // 2. Resolve target release.
    let tag = version.unwrap_or("latest");
    let release = fetch_release(REPO, tag)
        .with_context(|| format!("Could not fetch release `{}` from {}", tag, REPO))?;
    let target_version = release.tag_name.trim_start_matches('v').to_string();
    let asset_name = combined_asset_name()?;
    let up_to_date = target_version == current_version;

    // 3. JSON-only path — emit the structured report and exit.
    if json {
        let action: &'static str = if check {
            "checked"
        } else if up_to_date && !force {
            "no-op"
        } else {
            "would-update"
        };
        let report = UpdateReport {
            current: current_version.clone(),
            target: target_version.clone(),
            up_to_date,
            action,
            install_dir: install_dir.display().to_string(),
            install_mode: install_mode.to_string(),
            asset: asset_name.clone(),
            repo: REPO.to_string(),
            release_url: release.html_url.clone(),
            error: None,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).context("Could not serialise update report")?
        );
        if check || (up_to_date && !force) {
            return Ok(());
        }
        // Fall through to actually perform the update. The final printed line
        // will be plain text — the JSON header above is enough for CI gates;
        // we don't try to keep stdout pure-JSON during a real install (the
        // progress bar from `download_to_file` would defeat that anyway).
    } else {
        println!();
        println!("{}", "ven update".bold());
        println!("  current : {}", current_version);
        println!("  target  : {} ({})", target_version, REPO);
        println!("  dir     : {}", install_dir.display());
        println!("  mode    : {}", install_mode);
        if let Some(url) = release.html_url.as_deref() {
            println!("  release : {}", url);
        }
        println!();
    }

    // 4. Up-to-date short-circuit.
    if up_to_date && !force {
        if !json {
            println!(
                "{} ven {} is already the latest release. Nothing to do.",
                "[ok]".green(),
                current_version
            );
        }
        return Ok(());
    }

    // 5. --check stops here — never touches disk.
    if check {
        if !json {
            println!(
                "{} a newer ven is available: {} -> {}",
                "[!]".yellow(),
                current_version,
                target_version
            );
            println!("    run `ven update` to apply.");
        }
        return Ok(());
    }

    // 6. Confirm (unless --yes or non-interactive).
    if !yes && !json && is_interactive() {
        let prompt = format!("Apply ven {} -> {}?", current_version, target_version);
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(true)
            .interact()
            .unwrap_or(false);
        if !confirmed {
            println!("Aborted.");
            // exit 2 = user-aborted, per the long_about contract
            std::process::exit(2);
        }
    }

    // 7. Pre-flight: can we write to install_dir? If not, escalate.
    if let Err(probe_err) = write_probe(&install_dir) {
        if reentry {
            return Err(anyhow!(
                "Re-launched with elevation but {} is still not writable: {}",
                install_dir.display(),
                probe_err
            ));
        }
        eprintln!();
        eprintln!(
            "  {} {} is not writable by this process: {}",
            "[!]".yellow(),
            install_dir.display(),
            probe_err
        );
        eprintln!("       relaunching elevated to update the system install...");
        return reexec_elevated(version, yes, force, json);
    }

    // 8. Locate the right asset.
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            let available = release
                .assets
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow!(
                "Release {} does not contain asset `{}` for your platform.\n\
                 Available assets: {}",
                release.tag_name,
                asset_name,
                available
            )
        })?
        .clone();

    // 9. Download + verify.
    let tmp = tempfile::TempDir::new().context("Could not create temp dir for update")?;
    let archive_path = tmp.path().join(&asset_name);
    download_to_file(&asset.browser_download_url, &archive_path, USER_AGENT)
        .with_context(|| format!("Failed to download {}", asset.browser_download_url))?;

    let sha_url = format!(
        "https://github.com/{}/releases/download/{}/SHA256SUMS",
        REPO, release.tag_name
    );
    match fetch_manifest_sha256(&sha_url, &asset_name) {
        Ok(expected) => {
            verify_sha256(&archive_path, &expected)
                .with_context(|| format!("SHA256 verification failed for {}", asset_name))?;
            println!(
                "  {} SHA256 verified ({})",
                "[ok]".green(),
                short_hex(&expected)
            );
        }
        Err(e) => {
            if !force {
                bail!(
                    "Could not verify integrity of {}.\n\
                     The release's SHA256SUMS manifest at {} was not reachable: {}.\n\
                     Pass --force to apply without verification (not recommended).",
                    asset_name,
                    sha_url,
                    e
                );
            }
            eprintln!(
                "  {} SHA256SUMS unavailable ({}); continuing because --force.",
                "[warn]".yellow(),
                e
            );
        }
    }

    // 10. Extract + locate binaries.
    let extract_dir = tmp.path().join("extract");
    std::fs::create_dir_all(&extract_dir)
        .with_context(|| format!("Could not create extraction dir {}", extract_dir.display()))?;
    extract_combined(&archive_path, &extract_dir)?;

    let new_ven = find_binary(&extract_dir, "ven")
        .with_context(|| format!("Could not locate `ven` inside {}", asset_name))?;
    // ven-launcher is preferred but missing-from-asset shouldn't kill the update;
    // older releases bundled only `ven`. We log a warning and still swap `ven`.
    let new_launcher = find_binary(&extract_dir, "ven-launcher").ok();

    // 11. Self-replace.
    let target_ven = install_dir.join(exe_name("ven"));
    let target_launcher = install_dir.join(exe_name("ven-launcher"));

    replace_in_place(&new_ven, &target_ven).with_context(|| {
        format!(
            "Failed to swap {} (the running binary)",
            target_ven.display()
        )
    })?;
    println!(
        "  {} ven {} -> {}",
        "[ok]".green(),
        current_version,
        target_version
    );

    if let Some(launcher_src) = new_launcher {
        replace_in_place(&launcher_src, &target_launcher)
            .with_context(|| format!("Failed to swap {}", target_launcher.display()))?;
        println!("  {} ven-launcher updated", "[ok]".green());
    } else {
        eprintln!(
            "  {} ven-launcher missing from release asset; left unchanged.",
            "[warn]".yellow()
        );
    }

    println!();
    println!("{}", "Updated.".bold().green());
    println!("Open a new terminal and run `ven --version` to confirm.");
    if cfg!(target_os = "windows") {
        println!(
            "  Note: {} files in {} are leftovers from this update — Windows can't",
            "*.exe.old".dimmed(),
            install_dir.display()
        );
        println!("  delete a running .exe in place. They're safe to remove next reboot.");
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// GitHub helpers
// -----------------------------------------------------------------------------

fn fetch_release(repo: &str, tag: &str) -> Result<GhRelease> {
    let url = if tag == "latest" {
        format!("https://api.github.com/repos/{}/releases/latest", repo)
    } else {
        let tag = if tag.starts_with('v') {
            tag.to_string()
        } else {
            format!("v{}", tag)
        };
        format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            repo, tag
        )
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Could not build HTTP client")?;
    let mut req = client.get(&url);
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            req = req.bearer_auth(token);
        }
    }
    let response = req
        .send()
        .with_context(|| format!("Network request to {} failed", url))?
        .error_for_status()
        .with_context(|| format!("GitHub returned an error for {}", url))?;
    let release: GhRelease = response
        .json()
        .with_context(|| format!("GitHub release JSON at {} was unparseable", url))?;
    Ok(release)
}

// -----------------------------------------------------------------------------
// Platform + asset naming
// -----------------------------------------------------------------------------

fn combined_asset_name() -> Result<String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        bail!("Self-update is not supported on this OS yet.");
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        bail!("Self-update is not supported on this architecture yet.");
    };
    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    Ok(format!("ven-{}-{}.{}", os, arch, ext))
}

fn exe_name(stem: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    }
}

fn classify_install_dir(dir: &Path) -> &'static str {
    let s = dir.to_string_lossy().to_lowercase().replace('\\', "/");
    if s.ends_with("/.ven/bin") {
        return "user";
    }
    if s.contains("/program files/ven") || s == "/usr/local/bin" || s.starts_with("/usr/bin") {
        return "system";
    }
    "portable"
}

// -----------------------------------------------------------------------------
// Filesystem helpers
// -----------------------------------------------------------------------------

fn write_probe(dir: &Path) -> Result<()> {
    if !dir.exists() {
        std::fs::create_dir_all(dir).with_context(|| {
            format!(
                "Install dir {} does not exist and we can't create it",
                dir.display()
            )
        })?;
    }
    let probe = dir.join(".ven-update-probe");
    std::fs::write(&probe, b"ok")
        .with_context(|| format!("Cannot write to {} (needs elevation)", dir.display()))?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

fn extract_combined(archive: &Path, dest: &Path) -> Result<()> {
    let name = archive.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.ends_with(".zip") {
        let file = std::fs::File::open(archive)
            .with_context(|| format!("Could not open {}", archive.display()))?;
        let mut zip = zip::ZipArchive::new(file)
            .with_context(|| format!("{} is not a valid zip", archive.display()))?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let outpath = dest.join(entry.mangled_name());
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if entry.is_file() {
                let mut out = std::fs::File::create(&outpath)
                    .with_context(|| format!("Could not write {}", outpath.display()))?;
                std::io::copy(&mut entry, &mut out)?;
                #[cfg(unix)]
                if let Some(mode) = entry.unix_mode() {
                    use std::os::unix::fs::PermissionsExt;
                    let _ =
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode));
                }
            } else {
                std::fs::create_dir_all(&outpath)?;
            }
        }
        Ok(())
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let file = std::fs::File::open(archive)
            .with_context(|| format!("Could not open {}", archive.display()))?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut tar = tar::Archive::new(decoder);
        tar.unpack(dest)
            .with_context(|| format!("Failed to extract {}", archive.display()))?;
        Ok(())
    } else {
        bail!(
            "Unsupported archive format: {} (expected .zip or .tar.gz)",
            name
        );
    }
}

fn find_binary(root: &Path, stem: &str) -> Result<PathBuf> {
    let target = exe_name(stem);
    for entry in walkdir::WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(&target)
        {
            return Ok(entry.into_path());
        }
    }
    Err(anyhow!(
        "Could not find `{}` inside extracted release asset under {}",
        target,
        root.display()
    ))
}

// -----------------------------------------------------------------------------
// Self-replace
// -----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn replace_in_place(src: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        // Delegate the .exe → .exe.old rename to `core::uninstaller` so
        // both `ven update` and `ven uninstall` share the same Windows
        // self-orphan dance — if you patch one, you patch the other.
        crate::core::uninstaller::self_orphan_windows_exe(target)?;
    }
    std::fs::copy(src, target).with_context(|| {
        format!(
            "Could not write new {} from {}",
            target.display(),
            src.display()
        )
    })?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_in_place(src: &Path, target: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    // POSIX unlink keeps the open file descriptor valid for any process that
    // already had the file open — including ourselves. Writing a new file at
    // the same path creates a brand-new inode.
    if target.exists() {
        std::fs::remove_file(target)
            .with_context(|| format!("Could not unlink {}", target.display()))?;
    }
    std::fs::copy(src, target)
        .with_context(|| format!("Could not write new {}", target.display()))?;
    let mut perms = std::fs::metadata(target)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(target, perms)
        .with_context(|| format!("Could not chmod 755 {}", target.display()))?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Elevation
// -----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn reexec_elevated(version: Option<&str>, yes: bool, force: bool, json: bool) -> Result<()> {
    let exe = std::env::current_exe().context("Could not resolve current exe for re-launch")?;
    let args = elevation_args(version, yes, force, json);
    // ps_quote: PowerShell single-quote escaping is `''` for an embedded `'`.
    let arg_array = args
        .iter()
        .map(|a| format!("'{}'", a.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    let exe_quoted = exe.display().to_string().replace('\'', "''");
    let ps_cmd = format!(
        "Start-Process -FilePath '{}' -Verb RunAs -ArgumentList @({}) -Wait",
        exe_quoted, arg_array
    );

    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .status()
        .context("Could not launch elevated PowerShell (UAC)")?;
    if !status.success() {
        bail!(
            "Elevated update did not succeed (PowerShell exit code {:?}). \
             If you cancelled the UAC prompt, re-run `ven update` from an \
             elevated terminal.",
            status.code()
        );
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn reexec_elevated(version: Option<&str>, yes: bool, force: bool, json: bool) -> Result<()> {
    let exe = std::env::current_exe().context("Could not resolve current exe for re-launch")?;
    let mut args: Vec<String> = vec!["--".to_string(), exe.display().to_string()];
    args.extend(elevation_args(version, yes, force, json));

    let status = std::process::Command::new("sudo")
        .args(&args)
        .status()
        .context("Could not launch `sudo` for elevated update. Is sudo installed?")?;
    if !status.success() {
        bail!(
            "Elevated update did not succeed (sudo exit code {:?}). \
             If you don't have sudo, re-run `ven update` as root.",
            status.code()
        );
    }
    Ok(())
}

fn elevation_args(version: Option<&str>, yes: bool, force: bool, json: bool) -> Vec<String> {
    let mut a: Vec<String> = vec!["update".into(), "--reentry".into(), "--yes".into()];
    let _ = yes; // already implied — non-interactive after elevation
    if force {
        a.push("--force".into());
    }
    if json {
        a.push("--json".into());
    }
    if let Some(v) = version {
        a.push("--version".into());
        a.push(v.into());
    }
    a
}

// -----------------------------------------------------------------------------
// Misc helpers
// -----------------------------------------------------------------------------

fn is_interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn short_hex(s: &str) -> String {
    if s.len() <= 12 {
        s.to_string()
    } else {
        format!("{}...", &s[..12])
    }
}
