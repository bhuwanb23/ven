use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::ruby_install::{fetch_ruby_release_versions, install_ruby, RubyDownloader};

pub struct RubyPlugin;

impl LanguagePlugin for RubyPlugin {
    fn name(&self) -> &str {
        "ruby"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let dl = RubyDownloader::new()?;
        install_ruby(&dl, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        RubyDownloader::new()?.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        RubyDownloader::new()?.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        fetch_ruby_release_versions()?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Could not determine latest Ruby release"))
    }
}
