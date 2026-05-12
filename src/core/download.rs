use anyhow::{anyhow, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::integrity;

/// Node.js downloader - handles downloading from nodejs.org with checksum verification
pub struct NodeDownloader {
    storage_root: PathBuf, // ~/.ven  (or %USERPROFILE%\.ven on Windows)
    cache_dir: PathBuf,    // ~/.ven/.cache
}

impl NodeDownloader {
    pub fn new() -> Result<Self> {
        // FIXED: use home dir (~/.ven) — works on every OS and every user account
        // Users can override with VEN_STORAGE_PATH env var if they want
        let storage_root = std::env::var("VEN_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("Cannot find home directory")
                    .join(".ven")
            });

        let cache_dir = storage_root.join(".cache");

        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    /// Get platform-specific archive info: (os_str, arch_str, extension)
    fn get_platform_info() -> Result<(&'static str, &'static str, &'static str)> {
        let os = if cfg!(target_os = "windows") {
            "win"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            return Err(anyhow!("Unsupported OS"));
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(anyhow!("Unsupported architecture"));
        };

        // Windows uses .zip, everything else uses .tar.gz
        let ext = if cfg!(target_os = "windows") {
            "zip"
        } else {
            "tar.gz"
        };

        Ok((os, arch, ext))
    }

    /// Build download URL for a specific version
    /// Example: https://nodejs.org/dist/v20.11.0/node-v20.11.0-win-x64.zip
    fn build_download_url(version: &str) -> Result<String> {
        let (os, arch, ext) = Self::get_platform_info()?;

        // Ensure version has 'v' prefix
        let v = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        let filename = format!("node-{}-{}-{}.{}", v, os, arch, ext);
        Ok(format!("https://nodejs.org/dist/{}/{}", v, filename))
    }

    /// Build checksum URL (SHASUMS256.txt lives next to the archives)
    fn build_checksum_url(version: &str) -> String {
        let v = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        format!("https://nodejs.org/dist/{}/SHASUMS256.txt", v)
    }

    /// Download a file with a progress bar
    fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let client = Client::new();
        let response = client.get(url).send()?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Download failed: HTTP {} for {}",
                response.status(),
                url
            ));
        }

        let total_size = response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );

        let mut file = std::fs::File::create(dest)?;
        let mut downloaded: u64 = 0;

        let bytes = response.bytes()?;
        for chunk in bytes.chunks(8192) {
            file.write_all(chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    /// Fetch expected SHA256 checksum from nodejs.org for this version
    fn fetch_checksum(version: &str) -> Result<String> {
        let url = Self::build_checksum_url(version);
        let client = Client::new();
        let text = client.get(&url).send()?.text()?;

        // Figure out the filename we downloaded (to match line in SHASUMS256.txt)
        let ver_clean = version.trim_start_matches('v');
        let filename = if cfg!(target_os = "windows") {
            format!("node-v{}-win-x64.zip", ver_clean)
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                format!("node-v{}-darwin-arm64.tar.gz", ver_clean)
            } else {
                format!("node-v{}-darwin-x64.tar.gz", ver_clean)
            }
        } else {
            format!("node-v{}-linux-x64.tar.gz", ver_clean)
        };

        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 && parts[1] == filename {
                return Ok(parts[0].to_string());
            }
        }

        Err(anyhow!("Checksum not found for {}", filename))
    }

    /// Download a Node.js version archive — returns path to the cached archive file
    pub fn download(&self, version: &str) -> Result<PathBuf> {
        println!(
            "{} Preparing to download Node {}...",
            "[ARROW]".cyan(),
            version.bold()
        );

        let url = Self::build_download_url(version)?;
        println!("{} URL: {}", "•".blue(), url);

        std::fs::create_dir_all(&self.cache_dir)?;

        let filename = url.split('/').last().unwrap_or("node.tar.gz");
        let cache_path = self.cache_dir.join(filename);

        if cache_path.exists() {
            println!("{} Using cached archive", "[OK]".green());
        } else {
            self.download_file(&url, &cache_path)?;
        }

        println!("{} Verifying checksum...", "•".blue());
        match Self::fetch_checksum(version) {
            Ok(expected) => match integrity::verify_sha256(&cache_path, &expected) {
                Ok(()) => integrity::print_checksum_ok(filename),
                Err(e) => {
                    let _ = std::fs::remove_file(&cache_path);
                    return Err(anyhow!(
                        "Checksum mismatch! Corrupted download removed. Try again.\n  {}",
                        e
                    ));
                }
            },
            Err(e) => integrity::print_checksum_unavailable(filename, &e.to_string()),
        }

        Ok(cache_path)
    }

    /// Get the installation directory for a specific version
    /// Layout: ~/.ven/node/20.11.0/   (no 'v' prefix in folder name)
    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        let ver_clean = version.trim_start_matches('v');
        self.storage_root.join("node").join(ver_clean)
    }

    /// Get the bin directory path for a specific version
    /// Windows: ~/.ven/node/20.11.0/          (node.exe is in root)
    /// Unix:    ~/.ven/node/20.11.0/bin/       (node is in bin/)
    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let install_dir = self.get_install_dir(version);

        if !install_dir.exists() {
            return Err(anyhow!(
                "Node {} is not installed. Run: ven install node {}",
                version,
                version
            ));
        }

        if cfg!(target_os = "windows") {
            // On Windows: node.exe sits directly in the extracted root
            if install_dir.join("node.exe").exists() {
                Ok(install_dir)
            } else {
                Err(anyhow!(
                    "Node {} binaries not found at {}",
                    version,
                    install_dir.display()
                ))
            }
        } else {
            // On Unix: node is inside bin/
            let bin_dir = install_dir.join("bin");
            if bin_dir.exists() {
                Ok(bin_dir)
            } else {
                Err(anyhow!(
                    "Node {} bin/ not found at {}",
                    version,
                    install_dir.display()
                ))
            }
        }
    }

    /// List all installed Node versions (newest first)
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let node_dir = self.storage_root.join("node");

        if !node_dir.exists() {
            return Ok(vec![]);
        }

        let mut versions = Vec::new();

        for entry in std::fs::read_dir(&node_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    // Folder names are like "20.11.0" (no v prefix)
                    // Quick sanity check: starts with a digit
                    if name_str
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        versions.push(name_str.to_string());
                    }
                }
            }
        }

        // Sort newest first
        versions.sort_by(|a, b| {
            let parse =
                |v: &str| -> Vec<u32> { v.split('.').filter_map(|n| n.parse().ok()).collect() };
            parse(b).cmp(&parse(a))
        });

        Ok(versions)
    }
}
