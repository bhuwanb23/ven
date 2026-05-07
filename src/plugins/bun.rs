use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::bun_install::{fetch_bun_release_versions, install_bun, BunDownloader};

pub struct BunPlugin;

impl LanguagePlugin for BunPlugin {
    fn name(&self) -> &str {
        "bun"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = BunDownloader::new()?;
        install_bun(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = BunDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = BunDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let versions = fetch_bun_release_versions()?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest Bun release"))
    }
}
