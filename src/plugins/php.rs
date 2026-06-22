use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::php_install::{fetch_php_release_versions, install_php, PhpDownloader};

pub struct PhpPlugin;

impl LanguagePlugin for PhpPlugin {
    fn name(&self) -> &str {
        "php"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = PhpDownloader::new()?;
        install_php(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = PhpDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = PhpDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let versions = fetch_php_release_versions()?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest PHP release"))
    }
}
