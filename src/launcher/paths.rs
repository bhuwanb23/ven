//! Resolve launcher CLI path hints against the current working directory.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Directory to pass to activation / `find_ven_toml` walking.
///
/// - No argument → current working directory.
/// - Relative paths → resolved against cwd (works no matter where `ven-launcher` is invoked from).
/// - Absolute paths → used as-is.
/// - If the path points at a **file** (e.g. `...\ven.toml`), search starts from its parent folder.
pub fn resolve_activation_start_dir(project: Option<&Path>) -> Result<PathBuf> {
    let cwd =
        std::env::current_dir().context(
            "ven-launcher: could not read current working directory (needed to resolve .\\relative paths)",
        )?;

    let base: PathBuf = match project {
        None => cwd,
        Some(p) if p.as_os_str().is_empty() => cwd,
        Some(p) => {
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                cwd.join(p)
            }
        }
    };

    Ok(start_directory_for_walk(base))
}

/// Search root for `find_ven_toml`: directory as-is; files use their parent; a path ending in
/// `ven.toml` (even if missing) uses the parent so we don't treat the file segment as a folder name.
fn start_directory_for_walk(base: PathBuf) -> PathBuf {
    let looks_like_ven_toml_file = base
        .file_name()
        .and_then(|n| n.to_str())
        == Some("ven.toml");

    if looks_like_ven_toml_file {
        return base
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(base);
    }

    if base.is_file() {
        return base
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(base);
    }

    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ven_toml_name_nonexistent_yields_parent() {
        assert_eq!(
            start_directory_for_walk(PathBuf::from("myapp/ven.toml")),
            PathBuf::from("myapp")
        );
    }

    #[test]
    fn directory_passed_through() {
        assert_eq!(
            start_directory_for_walk(PathBuf::from("myapp/nested")),
            PathBuf::from("myapp/nested")
        );
    }
}
