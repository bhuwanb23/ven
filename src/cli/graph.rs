//! `ven graph` — inspect dependency intelligence state for the project.

use crate::core::load_config;
use crate::intelligence::conflicts::{analyze_npm_graph, engine_checks};
use crate::intelligence::display::{graph_to_json, print_full_intel_tree};
use crate::intelligence::engine::DependencyIntelligenceService;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;

pub fn cmd_graph(json: bool, resolve: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let key = DependencyIntelligenceService::project_key(&cwd);
    let mut root_packages: Vec<String> = cfg.packages.keys().cloned().collect();
    root_packages.sort();

    if !resolve {
        if let Some(snapshot) = DependencyIntelligenceService::load_snapshot(&key)? {
            if json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                print_graph_report(
                    project_name,
                    &cfg,
                    &snapshot.graph,
                    &snapshot.conflict_chains,
                    &snapshot.engine_incompatibilities,
                    &snapshot.warnings,
                    &root_packages,
                );
            }
            return Ok(());
        }
    }

    let graph = DependencyIntelligenceService::environment_graph(&cfg, &cfg.packages)?;
    let (conflict_chains, _suggestions) = analyze_npm_graph(&graph, &cfg.packages);
    let engine_incompatibilities = engine_checks(&graph);
    let warnings: Vec<String> = Vec::new();

    if json {
        println!("{}", graph_to_json(&graph)?);
    } else {
        print_graph_report(
            project_name,
            &cfg,
            &graph,
            &conflict_chains,
            &engine_incompatibilities,
            &warnings,
            &root_packages,
        );
    }

    Ok(())
}

fn print_graph_report(
    project_name: &str,
    cfg: &crate::core::config::VenConfig,
    graph: &crate::intelligence::graph::IntelGraph,
    conflict_chains: &[crate::intelligence::graph::ConflictChain],
    engine_incompatibilities: &[crate::intelligence::graph::EngineIncompatibility],
    warnings: &[String],
    root_packages: &[String],
) {
    println!("Dependency Graph: {}", project_name);
    println!("Runtime: {}", resolve_runtime_label(cfg));
    println!("");

    let conflict_packages = build_conflict_set(conflict_chains, engine_incompatibilities);
    let orphan_packages = find_orphan_packages(graph, root_packages);

    print_full_intel_tree(graph, root_packages, &conflict_packages, &orphan_packages);

    println!("");
    println!(
        "Conflicts: {}",
        conflict_chains.len() + engine_incompatibilities.len()
    );
    println!(
        "Warnings: {}{}",
        warnings.len(),
        summarize_warnings(warnings)
    );
    println!("Orphans: {}", orphan_packages.len());
}

fn resolve_runtime_label(cfg: &crate::core::config::VenConfig) -> String {
    if !cfg.runtime.node.is_empty() {
        format!("Node {}", cfg.runtime.node)
    } else if !cfg.runtime.bun.is_empty() {
        format!("Bun {}", cfg.runtime.bun)
    } else if !cfg.runtime.python.is_empty() {
        format!("Python {}", cfg.runtime.python)
    } else if !cfg.runtime.go.is_empty() {
        format!("Go {}", cfg.runtime.go)
    } else if !cfg.runtime.rust.is_empty() {
        format!("Rust {}", cfg.runtime.rust)
    } else if !cfg.runtime.java.is_empty() {
        format!("Java {}", cfg.runtime.java)
    } else if !cfg.runtime.deno.is_empty() {
        format!("Deno {}", cfg.runtime.deno)
    } else if !cfg.runtime.ruby.is_empty() {
        format!("Ruby {}", cfg.runtime.ruby)
    } else {
        "unknown".to_string()
    }
}

fn build_conflict_set(
    conflict_chains: &[crate::intelligence::graph::ConflictChain],
    engine_incompatibilities: &[crate::intelligence::graph::EngineIncompatibility],
) -> HashSet<String> {
    let mut set = HashSet::new();
    for chain in conflict_chains {
        set.insert(chain.package.clone());
    }
    for inc in engine_incompatibilities {
        set.insert(inc.package.clone());
    }
    set
}

fn find_orphan_packages(
    graph: &crate::intelligence::graph::IntelGraph,
    root_packages: &[String],
) -> HashSet<String> {
    let root_set: HashSet<String> = root_packages.iter().cloned().collect();
    let mut has_parent = HashSet::new();

    for edge in &graph.edges {
        if edge.kind != crate::intelligence::graph::EdgeKind::Dependency {
            continue;
        }
        if let Some((to_pkg, _)) = edge.to.rsplit_once('@') {
            has_parent.insert(to_pkg.to_string());
        }
    }

    graph
        .nodes
        .keys()
        .filter(|name| !root_set.contains(*name) && !has_parent.contains(*name))
        .cloned()
        .collect()
}

fn summarize_warnings(warnings: &[String]) -> String {
    if warnings.is_empty() {
        String::new()
    } else if warnings.iter().any(|w| w.to_lowercase().contains("cve")) {
        format!(" ({})", "CVE")
    } else {
        String::new()
    }
}
