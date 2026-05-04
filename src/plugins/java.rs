use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::java_install::{fetch_java_release_versions, install_java, JavaDownloader};

pub struct JavaPlugin;

impl LanguagePlugin for JavaPlugin {
    fn name(&self) -> &str {
        "java"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = JavaDownloader::new()?;
        install_java(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = JavaDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = JavaDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let versions = fetch_java_release_versions()?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest Java release"))
    }
}
