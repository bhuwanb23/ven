use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::rust_install::{fetch_rust_release_versions, install_rust, RustDownloader};

pub struct RustPlugin;

impl LanguagePlugin for RustPlugin {
    fn name(&self) -> &str {
        "rust"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = RustDownloader::new()?;
        install_rust(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = RustDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = RustDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let versions = fetch_rust_release_versions()?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest Rust release"))
    }
}
