//! Constraint verification and conflict chain construction.

use crate::intelligence::graph::{
    ConflictChain, IntelGraph, ResolutionAction, ResolutionOption,
};
use semver::{Version, VersionReq};
use std::collections::HashMap;

pub fn node_engine_satisfies(node_runtime: &str, requirement: &str) -> bool {
    let req = requirement.trim();
    if req == "*" || req.is_empty() {
        return true;
    }
    let node_major = node_runtime
        .split('.')
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(0);
    let min_ver: String = req.chars().skip_while(|c| !c.is_ascii_digit()).collect();
    let min_major = min_ver
        .split('.')
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(0);
    node_major >= min_major
}

pub fn version_satisfies_constraint(installed: &str, constraint: &str) -> bool {
    if constraint == "*" || constraint.is_empty() {
        return true;
    }
    let Ok(v) = Version::parse(installed.trim_start_matches('v')) else {
        return false;
    };
    if let Ok(req) = VersionReq::parse(constraint) {
        return req.matches(&v);
    }
    Version::parse(constraint.trim_start_matches('v'))
        .map(|exact| v == exact)
        .unwrap_or(false)
}

use crate::intelligence::graph::EdgeKind;
use crate::intelligence::graph::EngineIncompatibility;

/// Peer edges + ven.toml pin mismatches.
pub fn analyze_npm_graph(
    graph: &IntelGraph,
    existing_packages: &HashMap<String, String>,
) -> (Vec<ConflictChain>, Vec<ResolutionOption>) {
    let mut chains = Vec::new();
    let mut option_id = 1u32;
    let mut suggestions = Vec::new();

    for edge in &graph.edges {
        if edge.kind != EdgeKind::Peer {
            continue;
        }
        let dep_pkg = edge
            .to
            .rsplit_once('@')
            .map(|(p, _)| p)
            .unwrap_or(edge.to.as_str());
        let Some(resolved) = graph.nodes.get(dep_pkg) else {
            chains.push(ConflictChain {
                steps: vec![format!(
                    "{} declares peer {} ({}) but it is not resolved in the graph",
                    edge.from, dep_pkg, edge.constraint
                )],
                package: dep_pkg.to_string(),
            });
            continue;
        };
        if !version_satisfies_constraint(&resolved.version, &edge.constraint) {
            chains.push(ConflictChain {
                steps: vec![
                    format!(
                        "{} requires peer {} satisfying {}",
                        edge.from, dep_pkg, edge.constraint
                    ),
                    format!(
                        "Resolved {}@{} does not satisfy {}",
                        dep_pkg, resolved.version, edge.constraint
                    ),
                ],
                package: dep_pkg.to_string(),
            });
            suggestions.push(ResolutionOption {
                id: option_id,
                label: format!(
                    "Change {} to satisfy {} (e.g. align with {})",
                    dep_pkg, edge.constraint, edge.constraint
                ),
                action: ResolutionAction::Downgrade {
                    package: dep_pkg.to_string(),
                    version: edge.constraint.clone(),
                },
            });
            option_id += 1;
        }
    }

    for (existing_name, existing_ver) in existing_packages {
        if let Some(n) = graph.nodes.get(existing_name) {
            if &n.version != existing_ver
                && !existing_ver.contains('*')
                && existing_ver != "latest"
                && !version_satisfies_constraint(&n.version, existing_ver)
            {
                chains.push(ConflictChain {
                    steps: vec![
                        format!("ven.toml pins {} = \"{}\"", existing_name, existing_ver),
                        format!(
                            "Simulated graph resolves {}@{}",
                            existing_name, n.version
                        ),
                        format!(
                            "Constraint \"{}\" is not satisfied by {}",
                            existing_ver, n.version
                        ),
                    ],
                    package: existing_name.clone(),
                });
            }
        }
    }

    suggestions.push(ResolutionOption {
        id: option_id,
        label: "Cancel and adjust ven.toml or package version".into(),
        action: ResolutionAction::Cancel,
    });

    (chains, suggestions)
}

pub fn engine_checks(graph: &IntelGraph) -> Vec<EngineIncompatibility> {
    let mut out = Vec::new();
    for (name, node) in &graph.nodes {
        if let Some(ref eng) = node.engines_node {
            if !node_engine_satisfies(&graph.runtime_version, eng) {
                out.push(EngineIncompatibility {
                    package: name.clone(),
                    version: node.version.clone(),
                    required_node: eng.clone(),
                    current_node: graph.runtime_version.clone(),
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_engine_satisfies_basic() {
        assert!(node_engine_satisfies("20.11.0", ">= 0.10.0"));
        assert!(node_engine_satisfies("18.0.0", ">= 14"));
        assert!(!node_engine_satisfies("16.0.0", ">= 18"));
        assert!(node_engine_satisfies("20.0.0", "*"));
    }

    #[test]
    fn version_satisfies_constraint_basic() {
        assert!(version_satisfies_constraint("4.18.2", "^4.0.0"));
        assert!(!version_satisfies_constraint("4.18.2", "^5.0.0"));
    }
}
