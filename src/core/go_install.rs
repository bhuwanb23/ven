use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::integrity;

#[derive(Debug, Clone)]
pub struct GoDownloader {
    storage_root: PathBuf,
    cache_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct GoRelease {
    version: String,
    stable: bool,
    #[serde(default)]
    files: Vec<GoFile>,
}

#[derive(Debug, Deserialize, Clone)]
struct GoFile {
    filename: String,
    #[serde(default)]
    sha256: String,
}

impl GoDownloader {
    pub fn new() -> Result<Self> {
        let storage_root = crate::core::ven_home::ven_home();
        let cache_dir = storage_root.join(".cache");
        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    fn platform_info() -> Result<(&'static str, &'static str, &'static str)> {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else {
            return Err(anyhow!("Unsupported OS for Go install"));
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "amd64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(anyhow!("Unsupported architecture for Go install"));
        };

        let ext = if cfg!(target_os = "windows") {
            "zip"
        } else {
            "tar.gz"
        };
        Ok((os, arch, ext))
    }

    fn archive_filename(version: &str) -> Result<String> {
        let ver = normalize_go_version(version);
        let (os, arch, ext) = Self::platform_info()?;
        Ok(format!("go{ver}.{os}-{arch}.{ext}"))
    }

    fn download_url(version: &str) -> Result<String> {
        Ok(format!(
            "https://go.dev/dl/{}",
            Self::archive_filename(version)?
        ))
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        self.storage_root
            .join("go")
            .join(normalize_go_version(version))
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let root = self.get_install_dir(version);
        let bin = root.join("bin");
        let marker = if cfg!(target_os = "windows") {
            bin.join("go.exe")
        } else {
            bin.join("go")
        };
        if marker.is_file() {
            Ok(bin)
        } else {
            Err(anyhow!(
                "Go {} is not installed. Run: ven install go {}",
                normalize_go_version(version),
                normalize_go_version(version)
            ))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        let go_dir = self.storage_root.join("go");
        if !go_dir.exists() {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(go_dir)? {
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

    pub fn download(&self, version: &str) -> Result<PathBuf> {
        fs::create_dir_all(&self.cache_dir)?;
        let url = Self::download_url(version)?;
        let name = Self::archive_filename(version)?;
        let dest = self.cache_dir.join(name);
        if dest.is_file() {
            return Ok(dest);
        }
        // Streaming download with timeouts + retry; see integrity::download_to_file
        // for why this replaced `Client::new().get(url).bytes()?` everywhere.
        integrity::download_to_file(&url, &dest, &integrity::installer_user_agent("go"))
            .with_context(|| format!("Failed to download {}", url))?;
        Ok(dest)
    }
}

pub fn fetch_go_release_versions() -> Result<Vec<String>> {
    let releases: Vec<GoRelease> = Client::new()
        .get("https://go.dev/dl/?mode=json&include=all")
        .send()
        .context("Cannot reach go.dev")?
        .error_for_status()?
        .json()
        .context("Failed to parse Go release list")?;
    let mut versions: Vec<String> = releases
        .into_iter()
        .filter(|r| r.stable)
        .map(|r| r.version.trim_start_matches("go").to_string())
        .collect();
    versions.sort_by(|a, b| version_cmp_parts(b, a));
    versions.dedup();
    Ok(versions)
}

pub fn resolve_go_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = spec.trim().trim_start_matches("go");
    if spec.eq_ignore_ascii_case("latest") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No Go versions listed on go.dev"));
    }
    let parts: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        let v = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        if available.iter().any(|a| a == &v) {
            return Ok(v);
        }
        return Err(anyhow!("Go {} not found on go.dev", v));
    }
    if parts.len() == 2 {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Go {}.z release found", spec));
    }
    if parts.len() == 1 && !spec.is_empty() {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Go {}.x.y release found", spec));
    }
    Err(anyhow!("Invalid Go version spec: {}", spec))
}

pub fn install_go(downloader: &GoDownloader, version: &str) -> Result<()> {
    let archive = downloader.download(version)?;
    let archive_filename = GoDownloader::archive_filename(version)?;

    match fetch_go_sha256(&archive_filename) {
        Ok(hex) => match integrity::verify_sha256(&archive, &hex) {
            Ok(()) => integrity::print_checksum_ok(&archive_filename),
            Err(e) => {
                let _ = fs::remove_file(&archive);
                return Err(anyhow!(
                    "Go archive checksum mismatch for {} ({}). Cached file removed; rerun.",
                    archive_filename,
                    e
                ));
            }
        },
        Err(e) => integrity::print_checksum_unavailable(&archive_filename, &e.to_string()),
    }

    let install_root = downloader.get_install_dir(version);
    if install_root.exists() {
        fs::remove_dir_all(&install_root)?;
    }
    fs::create_dir_all(&install_root)?;
    extract_go_archive(&archive, &install_root)?;

    let bin_dir = downloader.get_bin_path(version)?;
    let go_bin = if cfg!(target_os = "windows") {
        bin_dir.join("go.exe")
    } else {
        bin_dir.join("go")
    };
    let smoke = integrity::smoke_test_binary(&go_bin, &["version"], "go version")
        .with_context(|| format!("go version smoke test failed at {}", go_bin.display()))?;
    integrity::print_smoke_ok(&smoke);
    Ok(())
}

/// Look up the SHA256 for `filename` from `https://go.dev/dl/?mode=json&include=all`.
fn fetch_go_sha256(filename: &str) -> Result<String> {
    let releases: Vec<GoRelease> = Client::new()
        .get("https://go.dev/dl/?mode=json&include=all")
        .send()
        .with_context(|| "Could not fetch go.dev release index")?
        .error_for_status()
        .with_context(|| "go.dev release index returned non-2xx")?
        .json()
        .with_context(|| "Failed to parse Go release index for checksum lookup")?;
    for r in releases {
        for f in r.files {
            if f.filename == filename {
                if f.sha256.len() == 64 && f.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Ok(f.sha256.to_ascii_lowercase());
                }
            }
        }
    }
    Err(anyhow!("no SHA256 entry for {} in go.dev index", filename))
}

fn extract_go_archive(archive_path: &Path, dest: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.mangled_name();
            let rel = name.strip_prefix("go").unwrap_or(&name);
            if rel.as_os_str().is_empty() {
                continue;
            }
            let outpath = dest.join(rel);
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
            let rel = path.strip_prefix("go").unwrap_or(&path).to_path_buf();
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

fn normalize_go_version(version: &str) -> String {
    version.trim().trim_start_matches("go").to_string()
}

fn version_cmp_parts(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|n| n.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };
    parse(a).cmp(&parse(b))
}
