//! Text rendering for intelligence graphs (verbose CLI output).

use crate::intelligence::graph::{IntelEdge, IntelGraph, IntelNode};
use colored::Colorize;
use std::collections::HashSet;

pub fn print_intel_tree(graph: &IntelGraph, root_name: &str) {
    let Some(root) = graph.first_node(root_name) else {
        println!("    {} (not in graph)", root_name.dimmed());
        return;
    };
    let mut visited: HashSet<String> = HashSet::new();
    print_node_recursive(graph, root, 0, true, &mut visited);
}

fn print_node_recursive(
    graph: &IntelGraph,
    node: &IntelNode,
    depth: u32,
    is_last: bool,
    visited: &mut HashSet<String>,
) {
    if !visited.insert(node.name.clone()) {
        return;
    }

    let indent = "  ".repeat(depth as usize);
    let connector = if depth == 0 {
        ""
    } else if is_last {
        "└─ "
    } else {
        "├─ "
    };

    // When this name resolves to multiple versions (diamond dep) we mark
    // the label so the user knows to investigate `ven why`.
    let name_version = if graph
        .nodes
        .get(&node.name)
        .map(|m| m.len() > 1)
        .unwrap_or(false)
    {
        let versions: Vec<String> = graph
            .nodes
            .get(&node.name)
            .map(|m| m.values().map(|n| n.version.clone()).collect())
            .unwrap_or_default();
        format!("{}@[{}]", node.name, versions.join(", "))
            .yellow()
            .bold()
            .to_string()
    } else {
        format!("{}@{}", node.name, node.version)
    };
    let mut meta = Vec::new();
    if let Some(ref d) = node.deprecated {
        meta.push(format!("deprecated: {}", d));
    }
    if let Some(ref eng) = node.engines_node {
        meta.push(format!("engines.node: {}", eng));
    }
    let meta_s = if meta.is_empty() {
        String::new()
    } else {
        format!(" ({})", meta.join(", ").dimmed())
    };

    println!("{}{}{}{}", indent, connector, name_version, meta_s);

    let children = direct_children(graph, &node.name);
    let n = children.len();
    for (i, child_name) in children.iter().enumerate() {
        if let Some(child) = graph.first_node(child_name) {
            print_node_recursive(graph, child, depth + 1, i + 1 == n, visited);
        }
    }
}

/// Follow dependency edges from `pkg@ver` ids.
fn direct_children(graph: &IntelGraph, package_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for IntelEdge { from, to, kind, .. } in &graph.edges {
        if *kind != crate::intelligence::graph::EdgeKind::Dependency {
            continue;
        }
        let Some((from_pkg, _)) = from.rsplit_once('@') else {
            continue;
        };
        if from_pkg != package_name {
            continue;
        }
        let Some((to_pkg, _)) = to.rsplit_once('@') else {
            continue;
        };
        if seen.insert(to_pkg.to_string()) {
            out.push(to_pkg.to_string());
        }
    }
    out.sort();
    out
}

pub fn print_full_intel_tree(
    graph: &IntelGraph,
    root_packages: &[String],
    conflict_packages: &HashSet<String>,
    orphan_packages: &HashSet<String>,
) {
    let mut roots: Vec<String> = root_packages
        .iter()
        .filter(|name| graph.nodes.contains_key(*name))
        .cloned()
        .collect();
    if roots.is_empty() {
        roots = graph.nodes.keys().cloned().collect();
        roots.sort();
    }

    for (i, root_name) in roots.iter().enumerate() {
        if let Some(root_node) = graph.first_node(root_name) {
            let is_last = i + 1 == roots.len();
            let mut visited = HashSet::new();
            print_node_recursive_full(
                graph,
                root_node,
                0,
                is_last,
                &mut visited,
                "",
                conflict_packages,
                orphan_packages,
            );
            if !is_last {
                println!("");
            }
        }
    }
}

fn print_node_recursive_full(
    graph: &IntelGraph,
    node: &IntelNode,
    depth: u32,
    is_last: bool,
    visited: &mut HashSet<String>,
    constraint: &str,
    conflict_packages: &HashSet<String>,
    orphan_packages: &HashSet<String>,
) {
    if !visited.insert(node.name.clone()) {
        return;
    }

    let indent = "  ".repeat(depth as usize);
    let connector = if depth == 0 {
        ""
    } else if is_last {
        "└─ "
    } else {
        "├─ "
    };

    let mut label = format!("{}@{}", node.name, node.version);
    if conflict_packages.contains(&node.name) {
        label = label.red().bold().to_string();
    }

    let mut extra = Vec::new();
    if !constraint.is_empty() {
        extra.push(constraint.to_string());
    }
    if orphan_packages.contains(&node.name) {
        extra.push("orphan".yellow().to_string());
    }
    if let Some(ref dep) = node.deprecated {
        extra.push(format!("deprecated: {}", dep).dimmed().to_string());
    }
    if let Some(ref eng) = node.engines_node {
        extra.push(format!("engines.node: {}", eng).dimmed().to_string());
    }

    let extras = if extra.is_empty() {
        String::new()
    } else {
        format!(" ({})", extra.join(", "))
    };

    println!("{}{}{}{}", indent, connector, label, extras);

    let children = direct_child_edges(graph, &node.name);
    let n = children.len();
    for (i, (child_name, child_constraint)) in children.into_iter().enumerate() {
        if let Some(child) = graph.first_node(&child_name) {
            print_node_recursive_full(
                graph,
                child,
                depth + 1,
                i + 1 == n,
                visited,
                &child_constraint,
                conflict_packages,
                orphan_packages,
            );
        }
    }
}

fn direct_child_edges(graph: &IntelGraph, package_name: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for IntelEdge {
        from,
        to,
        constraint,
        kind,
    } in &graph.edges
    {
        if *kind != crate::intelligence::graph::EdgeKind::Dependency {
            continue;
        }
        let Some((from_pkg, _)) = from.rsplit_once('@') else {
            continue;
        };
        if from_pkg != package_name {
            continue;
        }
        let Some((to_pkg, _)) = to.rsplit_once('@') else {
            continue;
        };
        if seen.insert(to_pkg.to_string()) {
            out.push((to_pkg.to_string(), constraint.clone()));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn print_intel_summary(graph: &IntelGraph) {
    let total: u64 = graph
        .nodes
        .values()
        .flat_map(|m| m.values())
        .filter_map(|n| n.size_bytes)
        .sum();
    println!(
        "    {} {} packages, {} edges, ~{} unpacked (where known)",
        "Summary:".dimmed(),
        graph.node_count(),
        graph.edges.len(),
        format_bytes(total)
    );
}

pub fn print_transitive_note(graph: &IntelGraph) {
    let max_depth = graph
        .nodes
        .values()
        .flat_map(|m| m.values())
        .map(|n| n.depth)
        .max()
        .unwrap_or(0);
    println!(
        "    {} max depth {}, {} unique packages",
        "Transitive:".dimmed(),
        max_depth,
        graph.node_count()
    );
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Flat text tree for `ven graph` without unicode box chars.
pub fn graph_to_text_tree(graph: &IntelGraph) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "runtime: {:?} {}",
        graph.runtime_kind, graph.runtime_version
    ));
    let mut names: Vec<_> = graph.nodes.keys().cloned().collect();
    names.sort();
    for name in names {
        if let Some(versions) = graph.nodes.get(&name) {
            for n in versions.values() {
                lines.push(format!("  - {}@{}", n.name, n.version));
            }
        }
    }
    lines.push(format!("  edges: {}", graph.edges.len()));
    lines.join("\n")
}

pub fn graph_to_json(graph: &IntelGraph) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(graph)
}
