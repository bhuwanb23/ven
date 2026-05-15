use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::integrity;

#[derive(Debug, Clone)]
pub struct RustDownloader {
    storage_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

impl RustDownloader {
    pub fn new() -> Result<Self> {
        let storage_root = crate::core::ven_home::ven_home();
        Ok(Self { storage_root })
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        self.storage_root
            .join("rust")
            .join(normalize_rust_version(version))
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let root = self.get_install_dir(version);
        let bin = root.join("bin");
        let marker = if cfg!(target_os = "windows") {
            bin.join("cargo.exe")
        } else {
            bin.join("cargo")
        };
        if marker.is_file() {
            Ok(bin)
        } else {
            Err(anyhow!(
                "Rust {} is not installed. Run: ven install rust {}",
                normalize_rust_version(version),
                normalize_rust_version(version)
            ))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        let rust_dir = self.storage_root.join("rust");
        if !rust_dir.exists() {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(rust_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        versions.push(name.to_string());
                    }
                }
            }
        }
        versions.sort_by(|a, b| version_cmp_parts(b, a));
        Ok(versions)
    }
}

pub fn fetch_rust_release_versions() -> Result<Vec<String>> {
    // lightweight source for recent versions; good enough for install suggestions
    let releases: Vec<GithubRelease> = Client::new()
        .get("https://api.github.com/repos/rust-lang/rust/releases?per_page=100")
        .header("User-Agent", "ven")
        .send()
        .context("Cannot reach GitHub for Rust releases")?
        .error_for_status()?
        .json()
        .context("Failed to parse Rust release list")?;
    let mut out: Vec<String> = releases
        .into_iter()
        .map(|r| r.tag_name.trim_start_matches('v').to_string())
        .filter(|v| {
            v.chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        })
        .collect();
    out.sort_by(|a, b| version_cmp_parts(b, a));
    out.dedup();
    Ok(out)
}

pub fn resolve_rust_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = normalize_rust_version(spec);
    if spec.eq_ignore_ascii_case("latest") || spec.eq_ignore_ascii_case("stable") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No Rust versions listed"));
    }
    let parts: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        let v = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        if available.iter().any(|a| a == &v) {
            return Ok(v);
        }
        return Err(anyhow!("Rust {} not found", v));
    }
    if parts.len() == 2 || parts.len() == 1 {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Rust release found matching {}", spec));
    }
    Err(anyhow!("Invalid Rust version spec: {}", spec))
}

pub fn install_rust(downloader: &RustDownloader, version: &str) -> Result<()> {
    let version = normalize_rust_version(version);
    let install_dir = downloader.get_install_dir(&version);
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;

    let rustup_bin = ensure_rustup_init(&install_dir)?;
    let mut cmd = Command::new(&rustup_bin);
    cmd.env("CARGO_HOME", &install_dir)
        .env("RUSTUP_HOME", &install_dir)
        .args([
            "-y",
            "--no-modify-path",
            "--profile",
            "minimal",
            "--default-toolchain",
            &version,
        ]);
    let status = cmd.status().context("Failed to execute rustup-init")?;
    if !status.success() {
        return Err(anyhow!("rustup-init failed to install Rust {}", version));
    }

    let bin = downloader.get_bin_path(&version)?;
    let cargo = if cfg!(target_os = "windows") {
        bin.join("cargo.exe")
    } else {
        bin.join("cargo")
    };
    let rustc = if cfg!(target_os = "windows") {
        bin.join("rustc.exe")
    } else {
        bin.join("rustc")
    };
    let cargo_smoke = integrity::smoke_test_binary(&cargo, &["--version"], "cargo")
        .with_context(|| format!("cargo --version smoke test failed at {}", cargo.display()))?;
    integrity::print_smoke_ok(&cargo_smoke);
    let rustc_smoke = integrity::smoke_test_binary(&rustc, &["--version"], "rustc")
        .with_context(|| format!("rustc --version smoke test failed at {}", rustc.display()))?;
    integrity::print_smoke_ok(&rustc_smoke);
    Ok(())
}

fn rustup_init_url() -> Result<&'static str> {
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Ok("https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe")
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "aarch64") {
        Ok("https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe")
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Ok("https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init")
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        Ok("https://static.rust-lang.org/rustup/dist/x86_64-apple-darwin/rustup-init")
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Ok("https://static.rust-lang.org/rustup/dist/aarch64-apple-darwin/rustup-init")
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        Ok("https://static.rust-lang.org/rustup/dist/aarch64-unknown-linux-gnu/rustup-init")
    } else {
        Err(anyhow!("Unsupported platform for rustup-init download"))
    }
}

fn ensure_rustup_init(install_root: &Path) -> Result<PathBuf> {
    let cache_dir = install_root.join(".cache");
    fs::create_dir_all(&cache_dir)?;
    #[cfg(target_os = "windows")]
    let filename = "rustup-init.exe";
    #[cfg(not(target_os = "windows"))]
    let filename = "rustup-init";

    let target = cache_dir.join(filename);
    if target.is_file() {
        return Ok(target);
    }

    let url = rustup_init_url()?;

    let resp = Client::new()
        .get(url)
        .send()
        .with_context(|| format!("Failed to download {}", url))?
        .error_for_status()?;
    fs::write(&target, resp.bytes()?)?;

    let sidecar = format!("{}.sha256", url);
    match integrity::fetch_sidecar_sha256(&sidecar) {
        Ok(hex) => match integrity::verify_sha256(&target, &hex) {
            Ok(()) => integrity::print_checksum_ok(filename),
            Err(e) => {
                let _ = fs::remove_file(&target);
                return Err(anyhow!(
                    "rustup-init checksum mismatch ({}). Cached file removed; rerun.",
                    e
                ));
            }
        },
        Err(e) => integrity::print_checksum_unavailable(filename, &e.to_string()),
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&target)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&target, perm)?;
    }

    Ok(target)
}

fn normalize_rust_version(v: &str) -> String {
    v.trim().trim_start_matches('v').to_string()
}

fn version_cmp_parts(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|n| n.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };
    parse(a).cmp(&parse(b))
}
