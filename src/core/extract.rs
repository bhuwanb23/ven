use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

const MAX_EXTRACT_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2GB

struct LimitWriter<W> {
    inner: W,
    written: u64,
}

impl<W: std::io::Write> LimitWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner, written: 0 }
    }
}

impl<W: std::io::Write> std::io::Write for LimitWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.written += n as u64;
        if self.written > MAX_EXTRACT_BYTES {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Decompression bomb: extracted size exceeds 2GB limit",
            ))
        } else {
            Ok(n)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Validate that `candidate` is within `base_dir` after canonicalization.
/// Prevents path traversal attacks (zip slip, tar path traversal) where
/// archive entries contain `../` components that could escape the
/// extraction directory.
pub fn validate_path_within_dir(candidate: &Path, base_dir: &Path) -> Result<PathBuf> {
    let canon_base = std::fs::canonicalize(base_dir)
        .unwrap_or_else(|_| base_dir.to_path_buf());
    let canon_candidate = std::fs::canonicalize(candidate)
        .unwrap_or_else(|_| candidate.to_path_buf());

    if canon_candidate.starts_with(&canon_base) {
        Ok(canon_candidate)
    } else {
        Err(anyhow!(
            "Path traversal detected: {} escapes extraction directory {}",
            candidate.display(),
            base_dir.display()
        ))
    }
}

/// Extract a ZIP archive (Windows)
#[cfg(target_os = "windows")]
fn extract_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    use std::fs::File;
    use zip::ZipArchive;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    println!("{} Extracting to {}...", "[ARROW]".cyan(), dest.display());

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let outpath = dest.join(entry.mangled_name());

        // Defense-in-depth: validate the resolved path is within dest.
        // mangled_name() already sanitizes, but we verify explicitly.
        validate_path_within_dir(&outpath, dest)?;

        // Basic symlink heuristic: skip files that look like symlinks
        // (zip crate doesn't have native symlink support)
        if !entry.is_dir() && entry.mangled_name().ends_with('/') {
            eprintln!(
                "{} Skipping potential symlink: {}",
                "[WARN]".yellow(),
                entry.mangled_name()
            );
            continue;
        }

        // Create parent directories
        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if entry.is_file() {
            let mut outfile = File::create(&outpath)?;
            let mut writer = LimitWriter::new(&mut outfile);
            std::io::copy(&mut entry, &mut writer)?;
            if writer.written > MAX_EXTRACT_BYTES {
                return Err(anyhow!("Decompression bomb: extracted size exceeds 2GB limit"));
            }
        } else {
            std::fs::create_dir_all(&outpath)?;
        }
    }

    Ok(())
}

/// Extract a TAR.GZ archive (Unix)
#[cfg(not(target_os = "windows"))]
fn extract_tar_gz(tar_path: &Path, dest: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;
    use tar::EntryType;

    let file = File::open(tar_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    println!("{} Extracting to {}...", "[ARROW]".cyan(), dest.display());

    // Use entries() iterator to validate each entry's path before unpacking.
    // The default unpack() does NOT protect against path traversal.
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.into_owned();
        let outpath = dest.join(&entry_path);

        // Reject path traversal attempts
        validate_path_within_dir(&outpath, dest)?;

        // Also reject absolute paths
        if entry_path.is_absolute() {
            return Err(anyhow!(
                "Refusing to extract absolute path: {}",
                entry_path.display()
            ));
        }

        // Check for symlinks/hardlinks and validate link target
        let entry_type = entry.header().entry_type();
        if entry_type == EntryType::Symlink || entry_type == EntryType::Link {
            let link_target = entry.link_name()?;
            if let Some(target) = link_target {
                let target_path = target.into_owned();
                let resolved_target = if target_path.is_relative() {
                    if let Some(parent) = outpath.parent() {
                        parent.join(&target_path)
                    } else {
                        target_path
                    }
                } else {
                    target_path
                };
                if let Err(e) = validate_path_within_dir(&resolved_target, dest) {
                    eprintln!(
                        "{} Skipping symlink {} -> {} ({})",
                        "[WARN]".yellow(),
                        entry_path.display(),
                        resolved_target.display(),
                        e
                    );
                    continue;
                }
            }
        }

        // Unpack individual entry with decompression bomb protection
        if entry_type.is_file() {
            let mut outfile = File::create(&outpath)?;
            let mut writer = LimitWriter::new(&mut outfile);
            std::io::copy(&mut entry, &mut writer)?;
            if writer.written > MAX_EXTRACT_BYTES {
                return Err(anyhow!("Decompression bomb: extracted size exceeds 2GB limit"));
            }
        } else if entry_type.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            // For other types (symlinks, etc.), fall back to unpack_in
            entry.unpack_in(dest)?;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
use std::fs::File;

/// Extract Node.js archive and move to proper location
pub fn extract_archive(archive_path: &Path, dest: &Path) -> Result<()> {
    // Create destination directory
    std::fs::create_dir_all(dest)?;

    #[cfg(target_os = "windows")]
    {
        extract_zip(archive_path, dest)?;

        // On Windows, the ZIP extracts to node-vX.Y.Z-win-x64/
        // We need to move contents to dest/
        let extracted_dir = find_extracted_dir(dest)?;
        if extracted_dir != *dest {
            move_contents(&extracted_dir, dest)?;
            std::fs::remove_dir_all(&extracted_dir)?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        extract_tar_gz(archive_path, dest)?;

        // On Unix, tar extracts to node-vX.Y.Z-os-arch/
        // We need to move contents to dest/
        let extracted_dir = find_extracted_dir(dest)?;
        if extracted_dir != *dest {
            move_contents(&extracted_dir, dest)?;
            std::fs::remove_dir_all(&extracted_dir)?;
        }
    }

    println!("{} Extraction complete", "[OK]".green());
    Ok(())
}

/// Find the actual extracted directory (node-vX.Y.Z-...)
fn find_extracted_dir(base: &Path) -> Result<PathBuf> {
    for entry in std::fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with("node-v") {
                return Ok(path);
            }
        }
    }

    // If no node-v* dir found, return base itself
    Ok(base.to_path_buf())
}

/// Move all contents from src to dest
fn move_contents(src: &Path, dest: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            move_contents(&src_path, &dest_path)?;
            std::fs::remove_dir(&src_path)?;
        } else {
            std::fs::rename(&src_path, &dest_path).or_else(|_| -> Result<()> {
                std::fs::copy(&src_path, &dest_path)?;
                std::fs::remove_file(&src_path)?;
                Ok(())
            })?;
        }
    }

    Ok(())
}

/// Install Node.js - download, extract, and setup
pub fn install_node(
    downloader: &crate::core::download::NodeDownloader,
    version: &str,
) -> Result<()> {
    println!(
        "{} Installing Node {}...",
        "[DOWNLOAD]".cyan(),
        version.bold()
    );

    // Download
    let archive_path = downloader.download(version)?;

    // Get installation directory
    let install_dir = downloader.get_install_dir(version);

    // Extract
    extract_archive(&archive_path, &install_dir)?;

    // Verify installation
    let bin_path = downloader.get_bin_path(version)?;

    #[cfg(target_os = "windows")]
    let node_binary = bin_path.join("node.exe");

    #[cfg(not(target_os = "windows"))]
    let node_binary = bin_path.join("node");

    if !node_binary.exists() {
        return Err(anyhow::anyhow!(
            "Installation verification failed: node binary not found at {}",
            node_binary.display()
        ));
    }

    let smoke = crate::core::integrity::smoke_test_binary(&node_binary, &["--version"], "v")?;
    crate::core::integrity::print_smoke_ok(&smoke);

    println!(
        "{} Node {} installed successfully",
        "[OK]".green(),
        version.bold()
    );
    println!("{} Binary: {}", "•".blue(), node_binary.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validate_path_within_dir_accepts_child() {
        let dir = tempdir().unwrap();
        let child = dir.path().join("subdir").join("file.txt");
        assert!(validate_path_within_dir(&child, dir.path()).is_ok());
    }

    #[test]
    fn validate_path_within_dir_rejects_traversal() {
        let dir = tempdir().unwrap();
        let traversal = dir.path().join("..").join("..").join("etc").join("passwd");
        assert!(validate_path_within_dir(&traversal, dir.path()).is_err());
    }

    #[test]
    fn validate_path_within_dir_rejects_absolute_escape() {
        let dir = tempdir().unwrap();
        let escape = PathBuf::from("/tmp/evil");
        assert!(validate_path_within_dir(&escape, dir.path()).is_err());
    }

    #[test]
    fn validate_path_within_dir_accepts_same_dir() {
        let dir = tempdir().unwrap();
        assert!(validate_path_within_dir(dir.path(), dir.path()).is_ok());
    }
}
