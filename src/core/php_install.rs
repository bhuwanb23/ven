use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::installer_base::{version_cmp_parts, BaseInstaller};
use crate::core::integrity;

#[derive(Debug, Clone)]
pub struct PhpDownloader {
    base: BaseInstaller,
}

impl PhpDownloader {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base: BaseInstaller::new()?,
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
            return Err(anyhow!("Unsupported OS for PHP install"));
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(anyhow!("Unsupported architecture for PHP install"));
        };

        let ext = if cfg!(target_os = "windows") {
            "zip"
        } else {
            "tar.gz"
        };
        Ok((os, arch, ext))
    }

    fn archive_filename(version: &str) -> Result<String> {
        let (os, arch, ext) = Self::platform_info()?;
        if os == "windows" {
            // PHP 8.3 and below use vs16, 8.4+ use vs17
            let vs = if version.starts_with("8.3.")
                || version.starts_with("8.2.")
                || version.starts_with("8.1.")
                || version.starts_with("8.0.")
            {
                "vs16"
            } else {
                "vs17"
            };
            Ok(format!("php-{}-nts-Win32-{}-{}.{}", version, vs, arch, ext))
        } else {
            Ok(format!("php-{}.{}", version, ext))
        }
    }

    fn download_url(version: &str) -> Result<String> {
        let (os, _, _) = Self::platform_info()?;
        if os == "windows" {
            Ok(format!(
                "https://downloads.php.net/~windows/releases/{}",
                Self::archive_filename(version)?
            ))
        } else {
            Ok(format!(
                "https://www.php.net/distributions/{}",
                Self::archive_filename(version)?
            ))
        }
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        self.base.get_install_dir("php", version)
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let root = self.get_install_dir(version);
        let marker = if cfg!(target_os = "windows") {
            root.join("php.exe")
        } else {
            root.join("bin").join("php")
        };
        if marker.is_file() {
            if cfg!(target_os = "windows") {
                Ok(root)
            } else {
                Ok(root.join("bin"))
            }
        } else {
            Err(anyhow!(
                "PHP {} is not installed. Run: ven install php {}",
                version,
                version
            ))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        self.base.list_installed("php")
    }

    pub fn download(&self, version: &str) -> Result<PathBuf> {
        fs::create_dir_all(&self.base.cache_dir)?;
        let url = Self::download_url(version)?;
        let name = Self::archive_filename(version)?;
        let dest = self.base.cache_dir.join(name);
        if dest.is_file() {
            return Ok(dest);
        }
        integrity::download_to_file(&url, &dest, &integrity::installer_user_agent("php"))
            .with_context(|| format!("Failed to download {}", url))?;
        Ok(dest)
    }
}

pub fn fetch_php_release_versions() -> Result<Vec<String>> {
    // Scrape the archives directory listing (has actual PHP binaries)
    let resp = Client::new()
        .get("https://downloads.php.net/~windows/releases/archives/")
        .send()
        .context("Cannot reach downloads.php.net")?
        .error_for_status()?;

    let body = resp.text().context("Failed to read PHP archives page")?;

    // Extract version numbers from the HTML
    // Files look like: php-8.3.31-nts-Win32-vs16-x64.zip
    let mut versions = Vec::new();
    for line in body.lines() {
        // Look for lines containing php-8.x.x-nts-Win32 (NTS binaries only)
        if line.contains("php-8.") && line.contains("-nts-Win32") && line.contains(".zip") {
            // Extract version: find "php-" then get version until next "-"
            if let Some(start) = line.find("php-8.") {
                let after_php = &line[start + 4..]; // skip "php-"
                if let Some(end) = after_php.find('-') {
                    let ver = &after_php[..end];
                    // Validate it's a proper version
                    if ver.chars().all(|c| c.is_ascii_digit() || c == '.') && ver.contains('.') {
                        versions.push(ver.to_string());
                    }
                }
            }
        }
    }

    versions.sort_by(|a, b| version_cmp_parts(b, a));
    versions.dedup();
    Ok(versions)
}

pub fn resolve_php_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("latest") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No PHP versions available"));
    }
    let parts: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        let v = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        if available.iter().any(|a| a == &v) {
            return Ok(v);
        }
        return Err(anyhow!("PHP {} not found", v));
    }
    if parts.len() == 2 {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No PHP {}.z release found", spec));
    }
    if parts.len() == 1 && !spec.is_empty() {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No PHP {}.x.y release found", spec));
    }
    Err(anyhow!("Invalid PHP version spec: {}", spec))
}

/// Fetch SHA256 checksum for a PHP release from php.net
fn fetch_php_checksum(version: &str) -> Result<String> {
    let (_, _, ext) = PhpDownloader::platform_info()?;
    let filename = format!("php-{}.{}", version, ext);

    // Try fetching the .sha256 sidecar file
    let sha_url = format!("https://www.php.net/distributions/{}.sha256", filename);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get(&sha_url)
        .send()
        .with_context(|| format!("Failed to fetch {}", sha_url))?;

    if !resp.status().is_success() {
        return Err(anyhow!("No checksum file at {}", sha_url));
    }

    let body = resp
        .text()
        .with_context(|| format!("Failed to read body of {}", sha_url))?;

    // Parse the SHA256 from the sidecar file (format: "hexhash  filename")
    for line in body.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2
            && parts[0].len() == 64
            && parts[0].chars().all(|c| c.is_ascii_hexdigit())
        {
            return Ok(parts[0].to_string());
        }
    }

    Err(anyhow!(
        "Could not parse SHA256 from checksum file for PHP {}",
        version
    ))
}

pub fn install_php(downloader: &PhpDownloader, version: &str) -> Result<()> {
    let archive = downloader.download(version)?;

    // Try to fetch and verify SHA256 checksum
    match fetch_php_checksum(version) {
        Ok(expected) => match integrity::verify_sha256(&archive, &expected) {
            Ok(()) => integrity::print_checksum_ok(&downloader.archive_filename(version)?),
            Err(e) => {
                fs::remove_file(&archive)?;
                return Err(anyhow!(
                    "PHP archive checksum mismatch for {} ({}). Cached file removed; rerun.",
                    version,
                    e
                ));
            }
        },
        Err(e) => {
            println!(
                "{} Checksum unavailable for PHP {} ({}). Continuing without verification.",
                "!".yellow(),
                version,
                e
            );
        }
    }

    let install_root = downloader.get_install_dir(version);
    if install_root.exists() {
        fs::remove_dir_all(&install_root)?;
    }
    fs::create_dir_all(&install_root)?;

    extract_php_archive(&archive, &install_root)?;

    let php_bin = if cfg!(target_os = "windows") {
        install_root.join("php.exe")
    } else {
        install_root.join("bin").join("php")
    };
    let smoke = integrity::smoke_test_binary(&php_bin, &["--version"], "PHP")
        .with_context(|| format!("php --version smoke test failed at {}", php_bin.display()))?;
    integrity::print_smoke_ok(&smoke);
    Ok(())
}

fn extract_php_archive(archive_path: &Path, dest: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.mangled_name();
            // PHP zip contains files directly (no top-level directory to strip)
            let outpath = dest.join(&name);
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
            // PHP tar contains files directly (no top-level directory to strip)
            let outpath = dest.join(&path);
            // Defense-in-depth: validate path stays within extraction directory
            super::extract::validate_path_within_dir(&outpath, dest)?;
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            e.unpack(outpath)?;
        }
    }
    Ok(())
}

use colored::Colorize;
