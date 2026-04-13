use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::LanguagePlugin;

pub struct NodePlugin;

impl LanguagePlugin for NodePlugin {
    fn name(&self) -> &str {
        "node"
    }

    // ── Install ────────────────────────────────────────────────────
    fn install_version(&self, version: &str) -> Result<()> {
        use crate::core::{install_node_native, NodeDownloader};

        let downloader = NodeDownloader::new()?;
        install_node_native(&downloader, version)
    }

    // ── List installed ─────────────────────────────────────────────
    fn list_installed(&self) -> Result<Vec<String>> {
        use crate::core::NodeDownloader;

        let downloader = NodeDownloader::new()?;
        downloader.list_installed()
    }

    // ── Bin path ───────────────────────────────────────────────────
    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        use crate::core::NodeDownloader;

        let downloader = NodeDownloader::new()?;
        downloader.get_bin_path(version)
    }

    // ── Latest version ─────────────────────────────────────────────
    fn latest_version(&self) -> Result<String> {
        // Fetch latest LTS from Node.js directly
        let response = reqwest::blocking::get("https://nodejs.org/dist/index.json")?;
        let releases: Vec<serde_json::Value> = response.json()?;

        // Find latest LTS version
        for release in releases {
            if let Some(lts) = release.get("lts") {
                if !lts.is_null() {
                    if let Some(version) = release.get("version").and_then(|v| v.as_str()) {
                        return Ok(version.trim_start_matches('v').to_string());
                    }
                }
            }
        }

        Err(anyhow!("Could not determine latest LTS version"))
    }
}
