use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::Colorize;
use crate::plugins::{NodePlugin, LanguagePlugin};
use crate::core::config::{find_ven_toml, parse_ven_toml};

#[derive(Parser)]
#[command(name = "ven")]
#[command(about = "Intelligent version and dependency manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a new runtime version
    Install {
        /// Language runtime (e.g., "node")
        #[arg(value_parser = ["node"])]
        runtime: String,

        /// Version to install (e.g., "20.11.0", "latest", "lts")
        version: String,
    },

    /// List installed versions
    List {
        /// Language runtime
        #[arg(value_parser = ["node"])]
        runtime: Option<String>,
    },

    /// Use a specific version globally
    Use {
        /// Language runtime
        #[arg(value_parser = ["node"])]
        runtime: String,

        /// Version to use
        version: String,
    },

    /// Show current configuration and active versions
    Current,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { runtime, version } => {
            handle_install(&runtime, &version)?;
        }
        Commands::List { runtime } => {
            handle_list(runtime.as_deref())?;
        }
        Commands::Use { runtime, version } => {
            handle_use(&runtime, &version)?;
        }
        Commands::Current => {
            handle_current()?;
        }
    }

    Ok(())
}

fn handle_install(runtime: &str, version: &str) -> Result<()> {
    match runtime {
        "node" => {
            let plugin = NodePlugin;
            
            // Resolve version aliases like "latest" or "lts"
            let resolved_version = if version == "latest" {
                println!("{} Resolving latest Node version...", "🔍".cyan());
                plugin.latest_version()?
            } else if version == "lts" {
                println!("{} Using LTS Node version...", "🌟".cyan());
                plugin.latest_version()? // For now, same as latest
            } else {
                version.to_string()
            };

            plugin.install_version(&resolved_version)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn handle_list(runtime: Option<&str>) -> Result<()> {
    match runtime.unwrap_or("node") {
        "node" => {
            let plugin = NodePlugin;
            let versions = plugin.list_installed()?;

            if versions.is_empty() {
                println!("{} No Node versions installed.", "⚠️".yellow());
                println!("Run: ven install node <version>");
            } else {
                println!("{} Installed Node versions:", "📦".blue());
                for v in versions {
                    println!("  {}", v);
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn handle_use(runtime: &str, version: &str) -> Result<()> {
    match runtime {
        "node" => {
            let plugin = NodePlugin;
            
            // Verify the version is installed
            let installed = plugin.list_installed()?;
            if !installed.contains(&version.to_string()) {
                return Err(anyhow::anyhow!(
                    "Node {} is not installed. Run: ven install node {}",
                    version, version
                ));
            }

            // Write to ~/.ven/versions/node file
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
            
            let versions_dir = home.join(".ven").join("versions");
            std::fs::create_dir_all(&versions_dir)?;
            
            let version_file = versions_dir.join("node");
            std::fs::write(&version_file, version)?;

            println!("{} Global Node version set to {}", "✓".green(), version.bold());
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn handle_current() -> Result<()> {
    // Find current directory's ven.toml if it exists
    let current_dir = std::env::current_dir()?;
    
    if let Some(toml_path) = find_ven_toml(&current_dir) {
        let config = parse_ven_toml(&toml_path)?;
        println!("{} Configuration from {}:", "📄".blue(), toml_path.display());
        
        if let Some(node_version) = &config.runtime.node {
            println!("  Node: {}", node_version.bold());
        }
        
        if let Some(packages) = &config.packages {
            if !packages.is_empty() {
                println!("  Packages:");
                for (pkg, ver) in packages {
                    println!("    {} = {}", pkg, ver);
                }
            }
        }
    } else {
        // No ven.toml, show global default
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        
        let global_version_file = home.join(".ven").join("versions").join("node");
        
        if global_version_file.exists() {
            let version = std::fs::read_to_string(&global_version_file)?;
            println!("{} Global Node version: {}", "🌍".blue(), version.trim().bold());
        } else {
            println!("{} No global Node version set.", "⚠️".yellow());
            println!("Run: ven use node <version>");
        }
    }

    Ok(())
}