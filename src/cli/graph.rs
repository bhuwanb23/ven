//! `ven graph` — inspect dependency intelligence state for the project.

use crate::core::load_config;
use crate::intelligence::display::{graph_to_json, graph_to_text_tree};
use crate::intelligence::engine::DependencyIntelligenceService;
use anyhow::Result;
use colored::Colorize;

pub fn cmd_graph(json: bool, resolve: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    let key = DependencyIntelligenceService::project_key(&cwd);

    if !resolve {
        if let Some(snapshot) = DependencyIntelligenceService::load_snapshot(&key)? {
            if json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("{}", "Last simulation snapshot".bold().cyan());
                println!(
                    "  compatible: {}",
                    if snapshot.compatible {
                        "yes".green()
                    } else {
                        "no".red()
                    }
                );
                println!("{}", graph_to_text_tree(&snapshot.graph));
            }
            return Ok(());
        }
    }

    let graph = DependencyIntelligenceService::environment_graph(&cfg, &cfg.packages)?;

    if json {
        println!("{}", graph_to_json(&graph)?);
    } else {
        if resolve {
            println!(
                "{}",
                "[INFO] Live manifest / node_modules snapshot (--resolve)".cyan()
            );
        } else {
            println!(
                "{}",
                "[TIP] No saved simulation — showing manifest/install snapshot. Use `ven graph --resolve` to skip cache, or run `ven check-add` / `ven add`."
                    .yellow()
            );
        }
        println!("{}", graph_to_text_tree(&graph));
    }

    Ok(())
}
