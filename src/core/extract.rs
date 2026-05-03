use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};

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

        // Create parent directories
        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if entry.is_file() {
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
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
    use tar::Archive;

    let file = File::open(tar_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    println!("{} Extracting to {}...", "[ARROW]".cyan(), dest.display());
    archive.unpack(dest)?;

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

    if node_binary.exists() {
        println!(
            "{} Node {} installed successfully",
            "[OK]".green(),
            version.bold()
        );
        println!("{} Binary: {}", "•".blue(), node_binary.display());
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Installation verification failed: node binary not found at {}",
            node_binary.display()
        ))
    }
}
