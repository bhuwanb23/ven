//! Text rendering for intelligence graphs (verbose CLI output).

use crate::intelligence::graph::{IntelEdge, IntelGraph, IntelNode};
use colored::Colorize;
use std::collections::HashSet;

pub fn print_intel_tree(graph: &IntelGraph, root_name: &str) {
    let Some(root) = graph.nodes.get(root_name) else {
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

    let name_version = format!("{}@{}", node.name, node.version);
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

    println!(
        "{}{}{}{}",
        indent,
        connector,
        name_version.bold(),
        meta_s
    );

    let children = direct_children(graph, &node.name);
    let n = children.len();
    for (i, child_name) in children.iter().enumerate() {
        if let Some(child) = graph.nodes.get(child_name) {
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

pub fn print_intel_summary(graph: &IntelGraph) {
    let total: u64 = graph
        .nodes
        .values()
        .filter_map(|n| n.size_bytes)
        .sum();
    println!(
        "    {} {} packages, {} edges, ~{} unpacked (where known)",
        "Summary:".dimmed(),
        graph.nodes.len(),
        graph.edges.len(),
        format_bytes(total)
    );
}

pub fn print_transitive_note(graph: &IntelGraph) {
    let max_depth = graph.nodes.values().map(|n| n.depth).max().unwrap_or(0);
    println!(
        "    {} max depth {}, {} unique packages",
        "Transitive:".dimmed(),
        max_depth,
        graph.nodes.len()
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
        if let Some(n) = graph.nodes.get(&name) {
            lines.push(format!("  - {}@{}", n.name, n.version));
        }
    }
    lines.push(format!("  edges: {}", graph.edges.len()));
    lines.join("\n")
}

pub fn graph_to_json(graph: &IntelGraph) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(graph)
}
