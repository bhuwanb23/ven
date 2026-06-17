use anyhow::Result;
use std::path::Path;

/// Recursively calculate directory size
pub fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                total_size += entry.metadata()?.len();
            } else if path.is_dir() {
                total_size += calculate_dir_size(&path)?;
            }
        }
    }

    Ok(total_size)
}

/// Calculate the total size of a package in node_modules
pub fn calculate_package_size(package: &str) -> Result<u64> {
    let pkg_path = std::env::current_dir()?.join("node_modules").join(package);

    if !pkg_path.exists() {
        return Ok(0);
    }

    let mut total_size = 0;

    for entry in std::fs::read_dir(&pkg_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            total_size += entry.metadata()?.len();
        } else if path.is_dir() {
            total_size += calculate_dir_size(&path)?;
        }
    }

    Ok(total_size)
}

/// Format bytes into human-readable string
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
