//! `ven check-add` — non-mutating dependency intelligence query.

use crate::core::load_config;
use crate::intelligence::display::print_intel_tree;
use crate::intelligence::engine::DependencyIntelligenceService;
use crate::intelligence::suggestions::print_conflict_report;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

pub fn cmd_check_add(package_specs: &[String], json: bool) -> Result<()> {
    if package_specs.is_empty() {
        println!("{}", "[ERROR] Specify at least one package".red());
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    let existing: HashMap<String, String> = cfg.packages.clone();

    for spec in package_specs {
        let (name, ver) = if spec.contains('@') && !spec.starts_with('@') {
            let parts: Vec<&str> = spec.splitn(2, '@').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (spec.clone(), "latest".to_string())
        };

        let check = DependencyIntelligenceService::check_add(&cfg, &name, &ver, &existing)?;
        let sim = DependencyIntelligenceService::simulate_add(&cfg, &name, &ver, &existing)?;

        if json {
            let out = serde_json::json!({
                "package": name,
                "version_spec": ver,
                "check_add": check,
                "simulation_compatible": sim.compatible,
                "graph_nodes": sim.graph.nodes.len(),
                "graph_edges": sim.graph.edges.len(),
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
            continue;
        }

        println!("\n{} {}", "ven check-add".bold().cyan(), name.bold());
        println!("  {} {}", "Stack:".dimmed(), check.stack_summary);
        if !check.warnings.is_empty() {
            for w in &check.warnings {
                println!("  {} {}", "[WARN]".yellow(), w);
            }
        }
        if let Some(ref r) = check.recommended {
            println!("  {} {}", "Resolved:".green(), r);
        }
        if !check.compatible_versions.is_empty() {
            println!(
                "  {} {}",
                "Sample engine-compatible versions:".dimmed(),
                check.compatible_versions.join(", ")
            );
        }
        if !check.incompatible_examples.is_empty() {
            println!("  {}", "Examples failing engines.node:".yellow());
            for (v, why) in &check.incompatible_examples {
                println!("    {} — {}", v.dimmed(), why);
            }
        }

        println!(
            "  {} {}",
            "Simulation:".dimmed(),
            if sim.compatible {
                "compatible".green()
            } else {
                "conflicts detected".red()
            }
        );
        print_conflict_report(&sim);
        println!("  {}", "Dependency tree (simulated):".dimmed());
        print_intel_tree(&sim.graph, &name);
    }

    Ok(())
}
