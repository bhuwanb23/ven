use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::installer_base::{version_cmp_parts, BaseInstaller};
use crate::core::integrity;

#[derive(Debug, Clone)]
pub struct JavaDownloader {
    base: BaseInstaller,
}

impl JavaDownloader {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base: BaseInstaller::new()?,
        })
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        self.base
            .get_install_dir("java", &version.trim().to_string())
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let root = self.get_install_dir(version);
        let bin = root.join("bin");
        let marker = if cfg!(target_os = "windows") {
            bin.join("java.exe")
        } else {
            bin.join("java")
        };
        if marker.is_file() {
            Ok(bin)
        } else {
            Err(anyhow!(
                "Java {} is not installed. Run: ven install java {}",
                version,
                version
            ))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        self.base.list_installed("java")
    }
}

pub fn fetch_java_release_versions() -> Result<Vec<String>> {
    // Fetch latest GA JDK releases per common feature lines from Adoptium.
    let mut out = Vec::new();
    for feature in [8_u32, 11, 17, 21, 22, 23] {
        let url = format!(
            "https://api.adoptium.net/v3/assets/feature_releases/{feature}/ga?architecture={}&image_type=jdk&jvm_impl=hotspot&os={}&page=0&page_size=10&project=jdk&vendor=eclipse",
            platform_arch(),
            platform_os()
        );
        let v: Value = Client::new()
            .get(&url)
            .send()
            .with_context(|| format!("Cannot reach adoptium for Java {}", feature))?
            .error_for_status()?
            .json()
            .context("Failed to parse Java release list")?;
        if let Some(arr) = v.as_array() {
            for item in arr {
                if let Some(semver) = item
                    .get("version_data")
                    .and_then(|x| x.get("semver"))
                    .and_then(|x| x.as_str())
                {
                    out.push(semver.to_string());
                }
            }
        }
    }
    out.sort_by(|a, b| version_cmp_parts(b, a));
    out.dedup();
    Ok(out)
}

pub fn resolve_java_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("latest") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No Java versions listed"));
    }
    let parts: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        let v = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        if available.iter().any(|a| a.starts_with(&v)) {
            return available
                .iter()
                .find(|a| a.starts_with(&v))
                .cloned()
                .ok_or_else(|| anyhow!("No Java {} found", v));
        }
        return Err(anyhow!("Java {} not found", v));
    }
    if !spec.contains('.') {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Java {}.x release found", spec));
    }
    let prefix = format!("{}.", spec);
    available
        .iter()
        .find(|v| v.starts_with(&prefix))
        .cloned()
        .ok_or_else(|| anyhow!("No Java release found matching {}", spec))
}

pub fn install_java(downloader: &JavaDownloader, version: &str) -> Result<()> {
    let (resolved_version, link, checksum) = resolve_download_link(version)?;
    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    fs::create_dir_all(&downloader.base.cache_dir)?;
    let archive_filename = format!("java-{}.{}", resolved_version, ext);
    let archive = downloader.base.cache_dir.join(&archive_filename);
    if !archive.is_file() {
        // Streaming download with timeouts + retry; see integrity::download_to_file
        // for why this replaced `Client::new().get(url).bytes()?` everywhere.
        // The old code's `.timeout(Duration::from_secs(600))` is no longer
        // needed — the shared helper uses per-read timeouts (60s between
        // chunks) instead of a total-request timeout, so genuinely slow
        // links work but stalled connections still fail fast.
        integrity::download_to_file(&link, &archive, &integrity::installer_user_agent("java"))
            .with_context(|| format!("Failed to download Java archive from {}", link))?;
    }

    if !checksum.is_empty() {
        match integrity::verify_sha256(&archive, &checksum) {
            Ok(()) => integrity::print_checksum_ok(&archive_filename),
            Err(e) => {
                let _ = fs::remove_file(&archive);
                return Err(anyhow!(
                    "Java archive checksum mismatch for {} ({}). Cached file removed; rerun.",
                    archive_filename,
                    e
                ));
            }
        }
    } else {
        let _ = fs::remove_file(&archive);
        return Err(anyhow!(
            "Checksum unavailable for {} — refusing to continue without verification.\n  \
             Reason: Adoptium API returned empty checksum\n  \
             Re-run the command when the network is available.",
            archive_filename
        ));
    }

    let install_dir = downloader.get_install_dir(&resolved_version);
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;
    extract_java_archive(&archive, &install_dir)?;

    let bin = downloader.get_bin_path(&resolved_version)?;
    let java_bin = if cfg!(target_os = "windows") {
        bin.join("java.exe")
    } else {
        bin.join("java")
    };
    let smoke = integrity::smoke_test_binary(&java_bin, &["--version"], "")
        .or_else(|_| integrity::smoke_test_binary(&java_bin, &["-version"], ""))
        .with_context(|| format!("java -version smoke test failed at {}", java_bin.display()))?;
    integrity::print_smoke_ok(&smoke);
    Ok(())
}

fn resolve_download_link(spec: &str) -> Result<(String, String, String)> {
    let major = spec
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(21);
    let url = format!(
        "https://api.adoptium.net/v3/assets/feature_releases/{major}/ga?architecture={}&image_type=jdk&jvm_impl=hotspot&os={}&page=0&page_size=20&project=jdk&vendor=eclipse",
        platform_arch(),
        platform_os()
    );
    let v: Value = Client::new()
        .get(&url)
        .send()
        .with_context(|| format!("Cannot reach adoptium for Java {}", major))?
        .error_for_status()?
        .json()
        .context("Failed to parse Java asset response")?;
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow!("Unexpected Java API format"))?;
    for item in arr {
        let semver = item
            .get("version_data")
            .and_then(|x| x.get("semver"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        if semver_matches(spec, semver) {
            let bin = item
                .get("binaries")
                .and_then(|x| x.as_array())
                .and_then(|x| x.first())
                .ok_or_else(|| anyhow!("No binary payload for Java {}", semver))?;
            let pkg = bin
                .get("package")
                .ok_or_else(|| anyhow!("No package payload for Java {}", semver))?;
            let link = pkg
                .get("link")
                .and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("No package link for Java {}", semver))?;
            let checksum = pkg
                .get("checksum")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            return Ok((semver.to_string(), link.to_string(), checksum));
        }
    }
    Err(anyhow!("No downloadable Java release found for {}", spec))
}

fn semver_matches(spec: &str, semver: &str) -> bool {
    if spec.eq_ignore_ascii_case("latest") {
        return true;
    }
    if !spec.contains('.') {
        return semver.starts_with(&format!("{}.", spec));
    }
    semver.starts_with(spec)
}

fn extract_java_archive(archive_path: &Path, dest: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.mangled_name();
            let rel = strip_first_component(&name);
            if rel.as_os_str().is_empty() {
                continue;
            }
            let outpath = dest.join(rel);
            // Defense-in-depth: validate the resolved path is within dest
            super::extract::validate_path_within_dir(&outpath, dest)?;
            if entry.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out = fs::File::create(&outpath)?;
                std::io::copy(&mut entry, &mut out)?;
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use flate2::read::GzDecoder;
        use std::fs::File;
        use tar::Archive;
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        for entry in archive.entries()? {
            let mut e = entry?;
            let path = e.path()?;
            let rel = strip_first_component(path.as_ref());
            if rel.as_os_str().is_empty() {
                continue;
            }
            let outpath = dest.join(rel);
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            e.unpack(outpath)?;
        }
    }
    Ok(())
}

fn strip_first_component(path: &Path) -> PathBuf {
    let mut c = path.components();
    let _ = c.next();
    c.as_path().to_path_buf()
}

fn platform_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "windows"
    }
}

fn platform_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    }
}
