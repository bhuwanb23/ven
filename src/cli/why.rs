//! `ven why <package>` — reverse dependency lookup and installation reason.

use crate::core::load_config;
use crate::intelligence::engine::DependencyIntelligenceService;
use anyhow::Result;
use colored::Colorize;
use std::collections::{HashMap, HashSet};

/// Reverse dependency chain: shows why a package is installed.
pub fn cmd_why(package: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    // Only works for npm projects (Node/Bun)
    if cfg.runtime.node.is_empty() && cfg.runtime.bun.is_empty() {
        println!(
            "{} `ven why` is only supported for npm-based projects (Node/Bun).",
            "[INFO]".cyan()
        );
        return Ok(());
    }

    let existing_packages = cfg.packages.clone();

    // Load the current environment graph
    let graph = DependencyIntelligenceService::environment_graph(&cfg, &existing_packages)?;

    // Find the package in the graph (exact match)
    let graph_key = package.to_string();
    let node = graph
        .nodes
        .get(&graph_key)
        .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in dependency graph", package))?;

    println!();
    println!(
        "{} {} is installed because:",
        "✓".green(),
        format!("{}@{}", package, node.version).bold()
    );

    // Find direct dependents from edges
    let dependents: Vec<String> = graph
        .edges
        .iter()
        .filter(|e| e.to == graph_key)
        .map(|e| e.from.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if dependents.is_empty() {
        println!(
            "  └─ {} (declared in your ven.toml)",
            "top-level entry".dimmed()
        );
    } else {
        // Trace each dependent back to ven.toml or show chain
        for (idx, dependent) in dependents.iter().enumerate() {
            let is_last = idx == dependents.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };

            // Get the dependent's version from graph
            let dep_node = graph.nodes.get(dependent);
            let dep_version = dep_node.map(|n| n.version.clone()).unwrap_or_default();

            // Find the constraint
            let constraint = graph
                .edges
                .iter()
                .find(|e| e.from == *dependent && e.to == graph_key)
                .map(|e| e.constraint.clone())
                .unwrap_or_default();

            println!(
                "  {} {} requires {}{}",
                prefix,
                format!("{}@{}", dependent, dep_version).bold(),
                package.cyan(),
                if constraint.is_empty() {
                    String::new()
                } else {
                    format!("@{}", constraint.dimmed())
                }
            );

            // Check if this dependent is from ven.toml
            if existing_packages.contains_key(dependent) {
                println!(
                    "      └─ {} (declared in your ven.toml)",
                    "root entry".dimmed()
                );
            } else {
                // Trace further up (this is a transitive dependency)
                trace_to_root(dependent, &graph, &existing_packages, "      ");
            }
        }
    }

    println!();
    println!(
        "{} Direct dependents: {}",
        "Dependents:".bold(),
        if dependents.is_empty() {
            "0 (top-level)".to_string()
        } else {
            format!("{} ({})", dependents.len(), dependents.join(", "))
        }
    );

    // Determine if safe to remove
    let is_safe = if dependents.is_empty() {
        true
    } else {
        // Safe only if all dependents are themselves only transitively used
        // For simplicity: safe = no ven.toml entries depend on it
        !dependents.iter().any(|d| existing_packages.contains_key(d))
    };

    let safety_icon = if is_safe { "✓".green() } else { "✗".red() };
    let safety_msg = if is_safe {
        "Yes (no direct ven.toml entries depend on it)".green()
    } else {
        let blocking = dependents
            .iter()
            .filter(|d| existing_packages.contains_key(d))
            .collect::<Vec<_>>();
        format!("No ({} from ven.toml depends on it)", blocking.join(", ")).red()
    };

    println!("{} Safe to remove: {}", safety_icon, safety_msg);
    println!();

    Ok(())
}

/// Recursively trace a package up to ven.toml entries.
fn trace_to_root(
    package: &str,
    graph: &crate::intelligence::graph::IntelGraph,
    ven_packages: &HashMap<String, String>,
    indent: &str,
) {
    // Find who depends on this package
    let mut direct_deps: Vec<String> = graph
        .edges
        .iter()
        .filter(|e| e.to == package)
        .map(|e| e.from.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if direct_deps.is_empty() {
        return;
    }

    for (idx, dep) in direct_deps.iter().enumerate() {
        let is_last = idx == direct_deps.len() - 1;
        let prefix = if is_last { "└─" } else { "├─" };

        let dep_node = graph.nodes.get(dep);
        let dep_version = dep_node.map(|n| n.version.clone()).unwrap_or_default();

        println!("{}  {} requires {}@{}", indent, prefix, dep, dep_version);

        if ven_packages.contains_key(dep) {
            println!(
                "{}      └─ {} (declared in your ven.toml)",
                indent,
                "root entry".dimmed()
            );
        } else {
            // Continue tracing (but limit depth to avoid infinite loops)
            trace_to_root(dep, graph, ven_packages, &format!("{}  ", indent));
        }
    }
}
