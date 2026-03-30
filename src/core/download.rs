use anyhow::{Result, anyhow};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use sha2::{Sha256, Digest};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Node.js downloader - handles downloading from nodejs.org with checksum verification
pub struct NodeDownloader {
    storage_root: PathBuf,  // D:\languages\node
    cache_dir: PathBuf,     // D:\languages\.cache
}

impl NodeDownloader {
    pub fn new() -> Result<Self> {
        // Use configurable storage path, default to D:\languages
        let storage_root = std::env::var("VEN_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(r"D:\languages"));
        
        let cache_dir = storage_root.join(".cache");
        
        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    /// Get platform-specific archive info
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
            if os == "darwin" {
                "arm64"  // Apple Silicon
            } else {
                "arm64"
            }
        } else {
            return Err(anyhow!("Unsupported architecture"));
        };

        let ext = if os == "win" {
            "zip"
        } else {
            "tar.gz"
        };

        Ok((os, arch, ext))
    }

    /// Build download URL for a specific version
    fn build_download_url(version: &str) -> Result<String> {
        let (os, arch, ext) = Self::get_platform_info()?;
        
        // Ensure version has 'v' prefix
        let version_with_v = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        let filename = match os {
            "win" => format!("node-{}-{}-{}.{}", version_with_v, os, arch, ext),
            _ => format!("node-{}-{}-{}.{}", version_with_v, os, arch, ext),
        };

        let url = format!("https://nodejs.org/dist/{}/{}", version_with_v, filename);
        Ok(url)
    }

    /// Build checksum URL
    fn build_checksum_url(version: &str) -> Result<String> {
        let version_with_v = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        
        Ok(format!(
            "https://nodejs.org/dist/{}/SHASUMS256.txt",
            version_with_v
        ))
    }

    /// Download file with progress bar
    fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        // Create parent directories
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let client = Client::new();
        let response = client.get(url).send()?;
        
        let total_size = response.content_length().unwrap_or(0);
        
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"));

        pb.set_message("Downloading");

        let mut file = std::fs::File::create(dest)?;
        let mut downloaded: u64 = 0;

        // Use bytes() instead of bytes_iter()
        let bytes = response.bytes()?;
        for chunk in bytes.chunks(8192) {
            file.write_all(chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    /// Fetch expected checksum from Node.js server
    fn fetch_checksum(version: &str) -> Result<String> {
        let checksum_url = Self::build_checksum_url(version)?;
        
        let client = Client::new();
        let response = client.get(checksum_url).send()?;
        let text = response.text()?;

        // Parse SHASUMS256.txt - format is: "hash  filename"
        let filename = if cfg!(target_os = "windows") {
            format!("node-v{}-win-x64.zip", version.trim_start_matches('v'))
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                format!("node-v{}-darwin-arm64.tar.gz", version.trim_start_matches('v'))
            } else {
                format!("node-v{}-darwin-x64.tar.gz", version.trim_start_matches('v'))
            }
        } else {
            format!("node-v{}-linux-x64.tar.gz", version.trim_start_matches('v'))
        };

        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 && parts[1] == filename {
                return Ok(parts[0].to_string());
            }
        }

        Err(anyhow!("Checksum not found for {}", filename))
    }

    /// Verify SHA256 checksum
    fn verify_checksum(file_path: &Path, expected: &str) -> Result<bool> {
        let mut file = std::fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        let actual = format!("{:x}", hasher.finalize());

        Ok(actual == expected)
    }

    /// Download a Node.js version - returns path to downloaded archive
    pub fn download(&self, version: &str) -> Result<PathBuf> {
        println!("{} Preparing to download Node {}...", "→".cyan(), version.bold());

        let url = Self::build_download_url(version)?;
        println!("{} URL: {}", "•".blue(), url);

        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;

        // Determine cache file path
        let filename = url.split('/').last().unwrap_or("node.tar.gz");
        let cache_path = self.cache_dir.join(filename);

        // Check if already cached
        if cache_path.exists() {
            println!("{} Using cached archive", "✓".green());
        } else {
            // Download
            self.download_file(&url, &cache_path)?;
        }

        // Verify checksum
        println!("{} Verifying checksum...", "•".blue());
        match Self::fetch_checksum(version) {
            Ok(expected) => {
                if Self::verify_checksum(&cache_path, &expected)? {
                    println!("{} Checksum verified", "✓".green());
                } else {
                    return Err(anyhow!("Checksum mismatch! File may be corrupted."));
                }
            }
            Err(e) => {
                println!("{} Warning: Could not verify checksum: {}", "!".yellow(), e);
                println!("{} Continuing without verification...", "•".blue());
            }
        }

        Ok(cache_path)
    }

    /// Get the installation directory for a version
    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        let version_clean = version.trim_start_matches('v');
        self.storage_root.join("node").join(format!("v{}", version_clean))
    }

    /// Get the bin directory for a version
    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let install_dir = self.get_install_dir(version);
        
        if !install_dir.exists() {
            return Err(anyhow!(
                "Node {} is not installed. Run: ven install node {}",
                version, version
            ));
        }

        // On Windows, binaries are in the root of install dir
        // On Unix, they're in bin/
        if cfg!(target_os = "windows") {
            if install_dir.join("node.exe").exists() {
                Ok(install_dir)
            } else {
                Err(anyhow!("Node {} binaries not found", version))
            }
        } else {
            let bin_dir = install_dir.join("bin");
            if bin_dir.exists() {
                Ok(bin_dir)
            } else {
                Err(anyhow!("Node {} binaries not found", version))
            }
        }
    }

    /// List all installed versions
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
                    if name_str.starts_with('v') {
                        // Remove 'v' prefix for consistency
                        versions.push(name_str.trim_start_matches('v').to_string());
                    }
                }
            }
        }

        // Sort by version number (newest first)
        versions.sort_by(|a, b| {
            let ver_a = semver::Version::parse(a).unwrap_or(semver::Version::new(0, 0, 0));
            let ver_b = semver::Version::parse(b).unwrap_or(semver::Version::new(0, 0, 0));
            ver_a.cmp(&ver_b).reverse()
        });

        Ok(versions)
    }
}
