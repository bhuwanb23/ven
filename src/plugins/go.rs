use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::go_install::{fetch_go_release_versions, install_go, GoDownloader};

pub struct GoPlugin;

impl LanguagePlugin for GoPlugin {
    fn name(&self) -> &str {
        "go"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = GoDownloader::new()?;
        install_go(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = GoDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = GoDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let versions = fetch_go_release_versions()?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest Go release"))
    }
}
