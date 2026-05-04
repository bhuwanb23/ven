use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DenoDownloader {
    storage_root: PathBuf,
    cache_dir: PathBuf,
}

impl DenoDownloader {
    pub fn new() -> Result<Self> {
        let storage_root = std::env::var("VEN_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().expect("Cannot find home directory").join(".ven"));
        let cache_dir = storage_root.join(".cache");
        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        self.storage_root
            .join("deno")
            .join(version.trim().trim_start_matches('v').to_string())
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let root = self.get_install_dir(version);
        let marker = if cfg!(target_os = "windows") {
            root.join("deno.exe")
        } else {
            root.join("deno")
        };
        if marker.is_file() {
            Ok(root)
        } else {
            Err(anyhow!(
                "Deno {} is not installed. Run: ven install deno {}",
                version,
                version
            ))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        let deno_dir = self.storage_root.join("deno");
        if !deno_dir.exists() {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(deno_dir)? {
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

pub fn fetch_deno_release_versions() -> Result<Vec<String>> {
    // Use GitHub tags API (no auth) for a reasonable list.
    let tags: Value = Client::new()
        .get("https://api.github.com/repos/denoland/deno/tags?per_page=100")
        .header("User-Agent", "ven")
        .send()
        .context("Cannot reach GitHub for deno tags")?
        .error_for_status()?
        .json()
        .context("Failed to parse deno tags list")?;
    let mut out = Vec::new();
    if let Some(arr) = tags.as_array() {
        for t in arr {
            if let Some(name) = t.get("name").and_then(|x| x.as_str()) {
                let v = name.trim_start_matches('v').to_string();
                if v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    out.push(v);
                }
            }
        }
    }
    out.sort_by(|a, b| version_cmp_parts(b, a));
    out.dedup();
    Ok(out)
}

pub fn resolve_deno_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = spec.trim().trim_start_matches('v');
    if spec.eq_ignore_ascii_case("latest") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No Deno versions listed"));
    }
    let parts: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 3 {
        let v = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        if available.iter().any(|a| a == &v) {
            return Ok(v);
        }
        return Err(anyhow!("Deno {} not found", v));
    }
    if parts.len() == 2 || parts.len() == 1 {
        let prefix = format!("{}.", spec);
        return available
            .iter()
            .find(|v| v.starts_with(&prefix))
            .cloned()
            .ok_or_else(|| anyhow!("No Deno release found matching {}", spec));
    }
    Err(anyhow!("Invalid Deno version spec: {}", spec))
}

pub fn install_deno(downloader: &DenoDownloader, version: &str) -> Result<()> {
    let version = version.trim().trim_start_matches('v');
    let url = build_download_url(version)?;
    fs::create_dir_all(&downloader.cache_dir)?;
    let archive = downloader
        .cache_dir
        .join(url.split('/').last().unwrap_or("deno.zip"));
    if !archive.is_file() {
        let resp = Client::new()
            .get(&url)
            .header("User-Agent", "ven")
            .send()
            .with_context(|| format!("Failed to download {}", url))?
            .error_for_status()?;
        fs::write(&archive, resp.bytes()?)?;
    }

    let install_dir = downloader.get_install_dir(version);
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;
    extract_deno_zip(&archive, &install_dir)?;
    let _ = downloader.get_bin_path(version)?;
    Ok(())
}

fn build_download_url(version: &str) -> Result<String> {
    let asset = if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        "deno-x86_64-pc-windows-msvc.zip"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "deno-x86_64-unknown-linux-gnu.zip"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "deno-x86_64-apple-darwin.zip"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "deno-aarch64-apple-darwin.zip"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "deno-aarch64-unknown-linux-gnu.zip"
    } else {
        return Err(anyhow!("Unsupported platform for Deno download"));
    };
    Ok(format!(
        "https://github.com/denoland/deno/releases/download/v{}/{}",
        version, asset
    ))
}

fn extract_deno_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    use std::fs::File;
    use zip::ZipArchive;
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let filename = Path::new(&name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if filename.is_empty() {
            continue;
        }
        // GitHub asset zips contain a single deno binary.
        let outpath = dest.join(filename);
        let mut out = fs::File::create(&outpath)?;
        std::io::copy(&mut entry, &mut out)?;
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = fs::metadata(&outpath)?.permissions();
            perm.set_mode(0o755);
            fs::set_permissions(&outpath, perm)?;
        }
    }
    Ok(())
}

fn version_cmp_parts(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|n| n.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };
    parse(a).cmp(&parse(b))
}

