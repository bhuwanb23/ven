//! Install MRI Ruby via:
//! - Windows: RubyInstaller2 `.7z` from GitHub (`oneclick/rubyinstaller2`).
//! - macOS/Linux: official build tarballs from `ruby/ruby-builder` releases (tags `ruby-X.Y.Z`).
use anyhow::{anyhow, Context, Result};
#[cfg(not(target_os = "windows"))]
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde_json::Value;
#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(not(target_os = "windows"))]
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "windows"))]
use tar::Archive;

use crate::core::integrity;

#[derive(Debug, Clone)]
pub struct RubyDownloader {
    storage_root: PathBuf,
    cache_dir: PathBuf,
}

impl RubyDownloader {
    pub fn new() -> Result<Self> {
        let storage_root = crate::core::ven_home::ven_home();
        let cache_dir = storage_root.join(".cache");
        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        self.storage_root
            .join("ruby")
            .join(normalize_ruby_semver(version))
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let root = self.get_install_dir(version);
        let exe = root.join("bin").join(if cfg!(target_os = "windows") {
            "ruby.exe"
        } else {
            "ruby"
        });
        if exe.is_file() {
            Ok(root.join("bin"))
        } else {
            Err(anyhow!(
                "Ruby {} is not installed. Run: ven install ruby {}",
                normalize_ruby_semver(version),
                normalize_ruby_semver(version),
            ))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        let dir = self.storage_root.join("ruby");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(dir)? {
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
        versions.sort_by(|a, b| version_cmp_parts_desc(b, a));
        Ok(versions)
    }
}

pub fn normalize_ruby_semver(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

pub fn ruby_gem_home_for_layout(install_root: &Path) -> PathBuf {
    let gems_base = install_root.join("lib").join("ruby").join("gems");
    if let Ok(rd) = fs::read_dir(&gems_base) {
        let mut dirs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        if let Some(p) = dirs.into_iter().last() {
            return p;
        }
    }
    install_root.join("gems")
}

/// Remote version list sorted newest-first (semver).
pub fn fetch_ruby_release_versions() -> Result<Vec<String>> {
    #[cfg(target_os = "windows")]
    {
        ri2_collect_versions_sorted()
    }
    #[cfg(not(target_os = "windows"))]
    {
        mri_github_builder_collect_versions_sorted()
    }
}

pub fn resolve_ruby_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = spec.trim().trim_start_matches('v');
    if spec.eq_ignore_ascii_case("latest") || spec.eq_ignore_ascii_case("lts") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No Ruby versions listed"));
    }
    let parts: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        let v = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        if available.iter().any(|a| a == &v) {
            return Ok(v);
        }
        return Err(anyhow!("Ruby {} not found in release list", v));
    }
    if parts.len() == 2 {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Ruby release found matching {}", spec));
    }
    if parts.len() == 1 && !spec.is_empty() {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Ruby release found for major {}", spec));
    }
    Err(anyhow!("Invalid Ruby version spec: {}", spec))
}

pub fn install_ruby(dl: &RubyDownloader, version: &str) -> Result<()> {
    let semver = normalize_ruby_semver(version);
    #[cfg(target_os = "windows")]
    {
        let (url, fname) = ri2_pick_asset_url(&semver)
            .ok_or_else(|| anyhow!("No RubyInstaller2 build found for {}", semver))?;
        fs::create_dir_all(&dl.cache_dir)?;
        let archive = dl.cache_dir.join(&fname);
        if !archive.is_file() {
            // Streaming download with timeouts + retry on transient errors.
            // Replaces the old `Client::new().get(url).send()?.bytes()?`
            // pattern that was buffering ~30 MB of RubyInstaller2 7z into
            // memory with no read timeout, so SSL-inspecting corporate
            // proxies (Zscaler / Netskope / Bluecoat) would stall mid-body
            // and surface as "error decoding response body / operation
            // timed out".
            integrity::download_to_file(&url, &archive, &integrity::installer_user_agent("ruby"))
                .with_context(|| format!("Failed to download {}", url))?;
        }
        verify_ruby_archive(&archive, &fname, &url);
        let install_dir = dl.get_install_dir(&semver);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }
        let staging_dir = tempfile::tempdir().context("temp dir for Ruby 7z")?;
        let unpack_base = staging_dir.path().join("ri2");
        fs::create_dir_all(&unpack_base)?;
        sevenz_rust::decompress_file(&archive, &unpack_base)
            .map_err(|e| anyhow!("7z unpack: {:?}", e))?;
        relocate_into_install_dir(&unpack_base, &install_dir)?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let url = ruby_builder_pick_asset_url(&semver)?;
        fs::create_dir_all(&dl.cache_dir)?;
        let fname = Path::new(&url)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ruby.tar.gz")
            .to_string();
        let archive = dl.cache_dir.join(&fname);
        if !archive.is_file() {
            // See Windows branch above for why we don't do `.bytes()?`.
            integrity::download_to_file(&url, &archive, &integrity::installer_user_agent("ruby"))
                .with_context(|| format!("Failed to download {}", url))?;
        }
        verify_ruby_archive(&archive, &fname, &url);
        let install_dir = dl.get_install_dir(&semver);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }
        let staging_dir = tempfile::tempdir().context("temp dir for Ruby tarball")?;
        let unpack_root = staging_dir.path().join("u");
        fs::create_dir_all(&unpack_root)?;
        extract_tar_gz(&archive, &unpack_root)?;
        relocate_into_install_dir(&unpack_root, &install_dir)?;
    }
    let bin_dir = dl.get_bin_path(&semver)?;
    let ruby_bin = if cfg!(target_os = "windows") {
        bin_dir.join("ruby.exe")
    } else {
        bin_dir.join("ruby")
    };
    let smoke = integrity::smoke_test_binary(&ruby_bin, &["--version"], "ruby")
        .with_context(|| format!("ruby --version smoke test failed at {}", ruby_bin.display()))?;
    integrity::print_smoke_ok(&smoke);
    Ok(())
}

/// Try to verify the Ruby archive's SHA256.
/// Windows: look for `SHA256SUMS.txt` next to the asset on the same release.
/// Unix: try `<url>.sha256` sidecar.
/// Either way, missing checksums degrade to a warning — Ruby's upstream sources
/// don't always publish per-asset hashes.
fn verify_ruby_archive(archive: &Path, filename: &str, url: &str) {
    // 1) sibling sidecar `<url>.sha256`
    let sidecar = format!("{}.sha256", url);
    if let Ok(hex) = integrity::fetch_sidecar_sha256(&sidecar) {
        return apply_ruby_checksum(archive, filename, &hex);
    }
    // 2) sibling SHA256SUMS.txt manifest (RubyInstaller2)
    if let Some(slash) = url.rfind('/') {
        let manifest = format!("{}/SHA256SUMS.txt", &url[..slash]);
        if let Ok(hex) = integrity::fetch_manifest_sha256(&manifest, filename) {
            return apply_ruby_checksum(archive, filename, &hex);
        }
    }
    integrity::print_checksum_unavailable(
        filename,
        "no SHA256 sidecar/manifest available upstream",
    );
}

fn apply_ruby_checksum(archive: &Path, filename: &str, hex: &str) {
    match integrity::verify_sha256(archive, hex) {
        Ok(()) => integrity::print_checksum_ok(filename),
        Err(e) => {
            let _ = fs::remove_file(archive);
            eprintln!(
                "[ERROR] Ruby checksum mismatch for {filename}: {e} — cached file removed; rerun."
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn extract_tar_gz(source: &Path, dest_parent: &Path) -> Result<()> {
    fs::create_dir_all(dest_parent)?;
    let file = fs::File::open(source)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest_parent)?;
    Ok(())
}

/// After `sevenz`/tar unpack into `staging_root`, normalize so `install_dir/bin/ruby*` exists.
fn relocate_into_install_dir(staging_root: &Path, install_dir: &Path) -> Result<()> {
    fs::create_dir_all(install_dir)?;
    fn has_layout(root: &Path) -> bool {
        root.join("bin")
            .join(if cfg!(target_os = "windows") {
                "ruby.exe"
            } else {
                "ruby"
            })
            .is_file()
    }
    if has_layout(staging_root) {
        copy_tree_recursive(staging_root, install_dir)?;
        return Ok(());
    }
    for entry in fs::read_dir(staging_root)? {
        let p = entry?.path();
        if p.is_dir() && has_layout(&p) {
            copy_tree_recursive(&p, install_dir)?;
            return Ok(());
        }
        if !p.is_dir() {
            continue;
        }
        // Nested (e.g. unpack created one folder holding another)
        for e2 in fs::read_dir(&p)? {
            let p2 = e2?.path();
            if p2.is_dir() && has_layout(&p2) {
                copy_tree_recursive(&p2, install_dir)?;
                return Ok(());
            }
        }
    }
    Err(anyhow!(
        "Unpack did not yield bin/{} — check upstream archive layout",
        if cfg!(target_os = "windows") {
            "ruby.exe"
        } else {
            "ruby"
        }
    ))
}

fn copy_tree_recursive(src: &Path, dst: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(src).min_depth(1).follow_links(false) {
        let entry = entry?;
        let meta = entry.path().symlink_metadata()?;
        let rel = entry.path().strip_prefix(src)?;
        let out_path = dst.join(rel);
        if meta.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else if meta.is_symlink() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let tgt = fs::read_link(entry.path())
                    .with_context(|| format!("read_link {}", entry.path().display()))?;
                if let Some(par) = out_path.parent() {
                    fs::create_dir_all(par)?;
                }
                let _ = fs::remove_file(&out_path);
                symlink(&tgt, &out_path)?;
            }
            #[cfg(not(unix))]
            {
                anyhow::bail!(
                    "Unexpected symlink inside Ruby bundle on Windows: {}",
                    entry.path().display()
                );
            }
        } else {
            if let Some(par) = out_path.parent() {
                fs::create_dir_all(par)?;
            }
            fs::copy(entry.path(), &out_path)?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn ruby_builder_pick_asset_url(semver: &str) -> Result<String> {
    let tag = format!("ruby-{semver}");
    let url = format!("https://api.github.com/repos/ruby/ruby-builder/releases/tags/{tag}");
    let json: Value = Client::new()
        .get(&url)
        .header("User-Agent", "ven")
        .send()
        .with_context(|| format!("Cannot GET ruby-builder tag {}", tag))?
        .error_for_status()
        .with_context(|| format!("Ruby {} not published on ruby/ruby-builder", semver))?
        .json()
        .context("Parse ruby-builder release JSON")?;
    pick_builder_asset_browser_url(&json, semver)
}

#[cfg(not(target_os = "windows"))]
fn pick_builder_asset_browser_url(release_json: &Value, semver: &str) -> Result<String> {
    let prefix = format!("ruby-{semver}-");
    let candidates: Vec<String> = release_json
        .get("assets")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("browser_download_url").and_then(|u| u.as_str()))
                .filter(|u| {
                    let name = Path::new(u)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    name.ends_with(".tar.gz") && name.starts_with(&prefix)
                })
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    if candidates.is_empty() {
        return Err(anyhow!("No tarball assets for Ruby {}", semver));
    }

    let prefers = platform_ruby_tarball_needles();

    for needle in &prefers {
        if let Some(u) = candidates.iter().find(|u| u.contains(*needle)) {
            return Ok(u.clone());
        }
    }

    candidates
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("No usable ruby-builder tarball asset"))
}

#[cfg(not(target_os = "windows"))]
fn platform_ruby_tarball_needles() -> Vec<&'static str> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        vec!["ubuntu-24.04-x64", "ubuntu-22.04-x64"]
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        vec!["ubuntu-24.04-arm64", "ubuntu-22.04-arm64"]
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        vec!["darwin-x64"]
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        vec!["darwin-arm64"]
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))] // uncommon host
    {
        vec![]
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
struct RiChoice {
    build: u32,
    url: String,
}

#[cfg(target_os = "windows")]
fn ri_suffix_for_arch() -> Result<&'static str> {
    if cfg!(target_arch = "x86_64") {
        Ok("x64")
    } else if cfg!(target_arch = "aarch64") {
        Ok("arm")
    } else {
        Err(anyhow!(
            "Unsupported Windows architecture for RubyInstaller2"
        ))
    }
}

#[cfg(target_os = "windows")]
fn ri2_collect_versions_sorted() -> Result<Vec<String>> {
    let suffix = ri_suffix_for_arch()?;
    let mut best: HashMap<String, RiChoice> = HashMap::new();
    gh_fetch_ri2_releases_into(&mut best, suffix)?;
    let mut sorted: Vec<String> = best.keys().cloned().collect();
    sorted.sort_by(|a, b| version_cmp_parts_desc(b, a));
    sorted.dedup();
    Ok(sorted)
}

#[cfg(target_os = "windows")]
fn gh_fetch_ri2_releases_into(
    acc: &mut HashMap<String, RiChoice>,
    arch_suffix: &str,
) -> Result<()> {
    let client = Client::new();
    for page in 1..=6u32 {
        let url = format!(
            "https://api.github.com/repos/oneclick/rubyinstaller2/releases?page={page}&per_page=100",
        );
        let arr: Value = client
            .get(&url)
            .header("User-Agent", "ven")
            .send()
            .context("RubyInstaller2 releases")?
            .error_for_status()?
            .json()
            .context("Parse RI2 releases JSON")?;
        let Some(list) = arr.as_array() else {
            break;
        };
        if list.is_empty() {
            break;
        }
        for rel in list {
            if let Some(avs) = rel.get("assets").and_then(|a| a.as_array()) {
                for a in avs {
                    let Some(name) = a.get("name").and_then(|n| n.as_str()) else {
                        continue;
                    };
                    let Some(download) = a.get("browser_download_url").and_then(|u| u.as_str())
                    else {
                        continue;
                    };
                    if !(name.ends_with(".7z") && !name.ends_with(".7z.asc")) {
                        continue;
                    }
                    if let Some((sem, build)) = parse_ri2_7z_name(name, arch_suffix) {
                        acc.entry(sem.clone())
                            .and_modify(|e| {
                                if build > e.build {
                                    *e = RiChoice {
                                        build,
                                        url: download.to_string(),
                                    };
                                }
                            })
                            .or_insert(RiChoice {
                                build,
                                url: download.to_string(),
                            });
                    }
                }
            }
        }
    }
    Ok(())
}

/// `rubyinstaller-4.0.3-1-x64.7z` → `("4.0.3", 1)` when `arch_suffix` is `x64`.
#[cfg(target_os = "windows")]
fn parse_ri2_7z_name(filename: &str, arch_suffix: &str) -> Option<(String, u32)> {
    let stem = filename
        .strip_prefix("rubyinstaller-")?
        .strip_suffix(".7z")?;
    let suf = format!("-{arch_suffix}");
    let mid = stem.strip_suffix(&suf)?;
    let idx = mid.rfind('-')?;
    let sem = mid[..idx].to_string();
    let build_str = &mid[idx + 1..];
    if sem.chars().next()?.is_ascii_digit() && sem.contains('.') {
        let build: u32 = build_str.parse().ok()?;
        return Some((sem, build));
    }
    None
}

#[cfg(target_os = "windows")]
fn ri2_pick_asset_url(semver: &str) -> Option<(String, String)> {
    let suffix = ri_suffix_for_arch().ok()?;
    let mut best: HashMap<String, RiChoice> = HashMap::new();
    gh_fetch_ri2_releases_into(&mut best, suffix).ok()?;
    best.remove(semver).map(|c| {
        let fname = Path::new(&c.url)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ruby.7z")
            .to_string();
        (c.url, fname)
    })
}

#[cfg(not(target_os = "windows"))]
fn mri_github_builder_collect_versions_sorted() -> Result<Vec<String>> {
    let client = Client::new();
    let mut seen = HashSet::new();
    for page in 1..=10u32 {
        let url = format!(
            "https://api.github.com/repos/ruby/ruby-builder/releases?page={}&per_page=100",
            page,
        );
        let arr: Value = client
            .get(&url)
            .header("User-Agent", "ven")
            .send()
            .context("ruby-builder releases page")?
            .error_for_status()?
            .json()
            .context("ruby-builder releases JSON")?;
        let Some(list) = arr.as_array() else {
            break;
        };
        if list.is_empty() {
            break;
        }
        for rel in list {
            let Some(tag) = rel.get("tag_name").and_then(|t| t.as_str()) else {
                continue;
            };
            let Some(rest) = tag.strip_prefix("ruby-") else {
                continue;
            };
            if rest.starts_with('-')
                || !rest
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                continue;
            }
            if rest.contains("preview") || rest.contains("dev") || rest.contains("rc") {
                continue;
            }
            if rest.chars().filter(|c| *c == '.').count() < 2 {
                continue;
            }
            seen.insert(rest.to_string());
        }
    }
    let mut out: Vec<String> = seen.into_iter().collect();
    out.sort_by(|a, b| version_cmp_parts_desc(b, a));
    Ok(out)
}

fn version_cmp_parts_desc(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split(|c| c == '.' || c == '-')
            .filter_map(|n| n.parse::<u32>().ok())
            .collect()
    };
    parse(a).cmp(&parse(b))
}
