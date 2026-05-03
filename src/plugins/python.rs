use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;
use crate::core::python_install::{
    fetch_python_release_versions, install_python, PythonDownloader,
};

pub struct PythonPlugin;

impl LanguagePlugin for PythonPlugin {
    fn name(&self) -> &str {
        "python"
    }

    fn install_version(&self, version: &str) -> Result<()> {
        let downloader = PythonDownloader::new()?;
        install_python(&downloader, version)
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        let downloader = PythonDownloader::new()?;
        downloader.list_installed()
    }

    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        let downloader = PythonDownloader::new()?;
        downloader.get_bin_path(version)
    }

    fn latest_version(&self) -> Result<String> {
        let v = fetch_python_release_versions()?;
        v.into_iter()
            .find(|x| x.starts_with("3."))
            .ok_or_else(|| anyhow!("Could not determine latest Python 3 release"))
    }
}
