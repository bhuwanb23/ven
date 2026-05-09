use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::deno_install::{fetch_deno_release_versions, install_deno, DenoDownloader};

pub struct DenoPlugin;

impl LanguagePlugin for DenoPlugin {
    fn name(&self) -> &str {
        "deno"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = DenoDownloader::new()?;
        install_deno(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = DenoDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = DenoDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let versions = fetch_deno_release_versions()?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest Deno release"))
    }
}
