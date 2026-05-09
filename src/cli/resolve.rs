//! `ven resolve` — automatic dependency conflict resolution.

use crate::cli::add::update_ven_toml_packages;
use crate::core::{load_config, npm_registry::NpmRegistry, packages};
use crate::intelligence::adapters::{find_highest_node_compatible_version, resolve_version};
use crate::intelligence::conflicts::{analyze_npm_graph, engine_checks};
use crate::intelligence::engine::DependencyIntelligenceService;
use crate::intelligence::graph::{EngineIncompatibility, ResolutionAction};
use anyhow::Result;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};

pub fn cmd_resolve() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg = load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    println!("\n  {} {}", "ven resolve".bold().cyan(), "[AUTO]".yellow());
    println!("  {} Scanning dependency graph...\n", "[INFO]".cyan());

    let graph = DependencyIntelligenceService::environment_graph(&cfg, &cfg.packages)?;
    let (conflict_chains, suggestions) = analyze_npm_graph(&graph, &cfg.packages);
    let engine_incompat = engine_checks(&graph);

    let conflict_entries = build_conflict_entries(&graph, &conflict_chains, &suggestions, &engine_incompat)?;
    let involved = collect_conflict_packages(&graph, &engine_incompat);
    let resolution_map = build_resolution_map(&graph, &cfg, &suggestions, &engine_incompat)?;

    if conflict_entries.is_empty() {
        println!("  {} No conflicts found. Your graph is already consistent.", "✓".green());
        return Ok(());
    }

    println!("  {} Found {} conflict(s):\n", "[INFO]".cyan(), conflict_entries.len());
    for (i, entry) in conflict_entries.iter().enumerate() {
        println!("    [{}] {}", i + 1, entry.summary);
        println!("      Fix: {}\n", entry.fix);
    }

    println!("  {}", "Optimal resolution found:".bold());
    let mut packages: Vec<String> = involved.into_iter().collect();
    packages.sort();
    for pkg in packages {
        if let Some(resolved) = resolution_map.get(&pkg) {
            println!(
                "    {}: {} → {}",
                pkg,
                resolved.current.dimmed(),
                resolved.suggested.green()
            );
        } else if let Some(node) = graph.nodes.get(&pkg) {
            println!("    {}: unchanged ({})", pkg, node.version.dimmed());
        } else {
            println!("    {}: unchanged", pkg);
        }
    }

    print!("\n  Apply? [y/N]: ");
    io::stdout().flush()?;
    let answer = io::stdin()
        .lock()
        .lines()
        .next()
        .and_then(|l| l.ok())
        .unwrap_or_default();

    if answer.trim().eq_ignore_ascii_case("y") {
        apply_resolution(&resolution_map)?;
    } else {
        println!("\n  {} No changes were applied.", "[INFO]".cyan());
    }

    println!();
    Ok(())
}

struct ConflictEntry {
    summary: String,
    fix: String,
}

struct ResolutionChange {
    package: String,
    current: String,
    suggested: String,
}

fn build_conflict_entries(
    graph: &crate::intelligence::graph::IntelGraph,
    conflict_chains: &[crate::intelligence::graph::ConflictChain],
    suggestions: &[crate::intelligence::graph::ResolutionOption],
    engine_incompat: &[EngineIncompatibility],
) -> Result<Vec<ConflictEntry>> {
    let mut entries = Vec::new();

    for chain in conflict_chains {
        let current = graph
            .nodes
            .get(&chain.package)
            .map(|n| n.version.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let fix = suggestion_hint_for_package(&chain.package, suggestions)?;
        entries.push(ConflictEntry {
            summary: format!("{}@{}", chain.package, current),
            fix,
        });
    }

    for inc in engine_incompat {
        let fix = if let Some(fixed) = find_highest_node_compatible_version(&inc.package, &graph.runtime_version)? {
            format!("{} → {}", inc.package, fixed)
        } else {
            format!("Downgrade {} to a Node-compatible release", inc.package)
        };

        entries.push(ConflictEntry {
            summary: format!("{}@{} ↔ Node {}", inc.package, inc.version, inc.required_node),
            fix,
        });
    }

    Ok(entries)
}

fn suggestion_hint_for_package(
    package: &str,
    suggestions: &[crate::intelligence::graph::ResolutionOption],
) -> Result<String> {
    let labels: Vec<String> = suggestions
        .iter()
        .filter_map(|opt| match &opt.action {
            ResolutionAction::Downgrade { package: pkg, version } if pkg == package => {
                Some(format!("{} → {}", package, version))
            }
            ResolutionAction::InstallVersion { package: pkg, version } if pkg == package => {
                Some(format!("{} → {}", package, version))
            }
            _ => None,
        })
        .collect();
    if !labels.is_empty() {
        Ok(labels.join(" OR "))
    } else {
        Ok("Adjust ven.toml or package version".to_string())
    }
}

fn collect_conflict_packages(
    graph: &crate::intelligence::graph::IntelGraph,
    engine_incompat: &[EngineIncompatibility],
) -> HashSet<String> {
    let mut packages = HashSet::new();

    for edge in &graph.edges {
        if edge.kind != crate::intelligence::graph::EdgeKind::Peer {
            continue;
        }
        let from_pkg = edge.from.rsplit_once('@').map(|(p, _)| p).unwrap_or(edge.from.as_str());
        let to_pkg = edge.to.rsplit_once('@').map(|(p, _)| p).unwrap_or(edge.to.as_str());
        packages.insert(from_pkg.to_string());
        packages.insert(to_pkg.to_string());
    }

    for inc in engine_incompat {
        packages.insert(inc.package.clone());
    }

    packages
}

fn build_resolution_map(
    graph: &crate::intelligence::graph::IntelGraph,
    cfg: &crate::core::config::VenConfig,
    suggestions: &[crate::intelligence::graph::ResolutionOption],
    engine_incompat: &[EngineIncompatibility],
) -> Result<HashMap<String, ResolutionChange>> {
    let mut map = HashMap::new();

    for opt in suggestions {
        if let ResolutionAction::Downgrade { package, version } = &opt.action {
            let current = graph
                .nodes
                .get(package)
                .map(|n| n.version.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let suggested = resolve_package_version(package, version)?;
            if suggested != current {
                map.insert(
                    package.clone(),
                    ResolutionChange {
                        package: package.clone(),
                        current,
                        suggested,
                    },
                );
            }
        } else if let ResolutionAction::InstallVersion { package, version } = &opt.action {
            let current = graph
                .nodes
                .get(package)
                .map(|n| n.version.clone())
                .unwrap_or_else(|| "unknown".to_string());
            if *version != current {
                map.insert(
                    package.clone(),
                    ResolutionChange {
                        package: package.clone(),
                        current,
                        suggested: version.clone(),
                    },
                );
            }
        }
    }

    for inc in engine_incompat {
        if map.contains_key(&inc.package) {
            continue;
        }
        let current = graph
            .nodes
            .get(&inc.package)
            .map(|n| n.version.clone())
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(suggested) = find_highest_node_compatible_version(&inc.package, &cfg.runtime.node)? {
            if suggested != current {
                map.insert(
                    inc.package.clone(),
                    ResolutionChange {
                        package: inc.package.clone(),
                        current,
                        suggested,
                    },
                );
            }
        }
    }

    Ok(map)
}

fn resolve_package_version(package: &str, version_spec: &str) -> Result<String> {
    let pkg = package.to_string();
    let spec = version_spec.to_string();
    let resolved = DependencyIntelligenceService::block_on(move || async move {
        let registry = NpmRegistry::new()?;
        let metadata = registry.fetch_package_metadata(&pkg).await?;
        resolve_version(&metadata, &spec)
    })?;
    Ok(resolved)
}

fn apply_resolution(changes: &HashMap<String, ResolutionChange>) -> Result<()> {
    if changes.is_empty() {
        println!("  {} Nothing to apply.", "[INFO]".cyan());
        return Ok(());
    }

    let mut updated_packages = Vec::new();
    for change in changes.values() {
        println!(
            "  {} Applying {} → {}",
            "[ACTION]".cyan(),
            change.package.bold(),
            change.suggested.green()
        );
        packages::npm_install(&change.package, &change.suggested)?;
        updated_packages.push((change.package.clone(), change.suggested.clone()));
    }

    update_ven_toml_packages(&updated_packages)?;
    println!("  {} Updated ven.toml with resolved package versions.", "[OK]".green());
    Ok(())
}
