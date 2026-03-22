use anyhow::{Result, anyhow};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use super::LanguagePlugin;

pub struct NodePlugin;

impl LanguagePlugin for NodePlugin {
    fn name(&self) -> &str { "node" }

    // ── Install ────────────────────────────────────────────────────
    fn install_version(&self, version: &str) -> Result<()> {
        println!("{} Installing Node {}...", "↓".cyan(), version.bold());

        // Call fnm as a subprocess
        let status = Command::new("fnm")
            .args(["install", version])
            .status()
            .map_err(|_| anyhow!("fnm not found. Install it: curl -fsSL https://fnm.vercel.app/install | bash"))?;

        if !status.success() {
            return Err(anyhow!("fnm failed to install Node {}. Check the version is valid.", version));
        }

        println!("{} Node {} installed", "✓".green(), version.bold());
        Ok(())
    }

    // ── List installed ─────────────────────────────────────────────
    fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("fnm")
            .args(["list"])
            .output()
            .map_err(|_| anyhow!("fnm not found"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // fnm list output looks like:
        // * v20.11.0 default
        // v22.3.0
        // v18.20.0
        let versions: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim().trim_start_matches('*').trim();
                // Extract version number from "v20.11.0 default" → "20.11.0"
                trimmed.split_whitespace().next()
                    .map(|v| v.trim_start_matches('v').to_string())
            })
            .filter(|v| !v.is_empty())
            .collect();

        Ok(versions)
    }

    // ── Bin path ───────────────────────────────────────────────────
    fn bin_path(&self, version: &str) -> Result<PathBuf> {
        // fnm stores Node binaries at:
        // ~/.fnm/node-versions/v<version>/installation/bin/
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Cannot find home directory"))?;

        let bin = home
            .join(".fnm")
            .join("node-versions")
            .join(format!("v{}", version))
            .join("installation")
            .join("bin");

        if !bin.exists() {
            return Err(anyhow!(
                "Node {} is not installed. Run: ven install node {}",
                version, version
            ));
        }

        Ok(bin)
    }

    // ── Latest version ─────────────────────────────────────────────
    fn latest_version(&self) -> Result<String> {
        // Ask fnm what the latest LTS is
        let output = Command::new("fnm")
            .args(["list-remote", "--lts"])
            .output()
            .map_err(|_| anyhow!("fnm not found"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let latest = stdout.lines().last()
            .map(|l| l.trim().trim_start_matches('v').to_string())
            .ok_or_else(|| anyhow!("Could not determine latest Node version"))?;

        Ok(latest)
    }
}