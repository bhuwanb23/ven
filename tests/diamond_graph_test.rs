//! Diamond-dependency regression tests for the multi-version `IntelGraph`.
//!
//! Before C-1 was fixed, the `NpmGraphBuilder` keyed the `nodes` map by
//! package name only, so a dep that resolved to two versions (e.g. `d@2`
//! via `b`, `d@3` via `c`) silently dropped the second visit. This file
//! tests the post-fix shape: both versions present, conflict surfaced, the
//! dep appears in two `required_by` slots.

use std::collections::BTreeMap;
use ven::intelligence::graph::{IntelGraph, IntelNode, RuntimeKind};

/// Build a hand-crafted `IntelGraph` representing the diamond
///
/// ```text
///     a
///    / \
///   b   c
///    \ / \
///     d@2 d@3   <-- diamond: d appears as 2.0.0 (via b) and 3.0.0 (via c)
/// ```
fn diamond_graph() -> IntelGraph {
    let mut nodes: BTreeMap<String, BTreeMap<semver::Version, IntelNode>> = BTreeMap::new();
    let mut insert = |name: &str, version: &str, required_by: Vec<String>| {
        let v = semver::Version::parse(version).unwrap();
        let node = IntelNode {
            name: name.into(),
            version: version.into(),
            depth: 0,
            dependencies: Default::default(),
            engines_node: None,
            deprecated: None,
            license: None,
            size_bytes: None,
            required_by,
            integrity: None,
        };
        nodes.entry(name.into()).or_default().insert(v, node);
    };
    insert("a", "1.0.0", vec![]);
    insert("b", "1.0.0", vec!["a@1.0.0".into()]);
    insert("c", "1.0.0", vec!["a@1.0.0".into()]);
    insert("d", "2.0.0", vec!["b@1.0.0".into()]);
    insert("d", "3.0.0", vec!["c@1.0.0".into()]);

    IntelGraph {
        runtime_kind: RuntimeKind::NpmFamily,
        runtime_version: "20".into(),
        nodes,
        edges: vec![],
    }
}

/// The headline C-1 assertion: the diamond must surface as a conflict.
#[test]
fn diamond_dep_is_detected_as_conflict() {
    let g = diamond_graph();
    assert!(g.has_version_conflicts(), "diamond must be detected");
    assert_eq!(g.conflicted_names(), vec!["d".to_string()]);
}

/// Both versions of `d` must be retrievable; the second visit must NOT
/// have clobbered the first (the pre-fix bug).
#[test]
fn diamond_keeps_both_versions() {
    let g = diamond_graph();
    let d2 = g
        .get_node("d", &semver::Version::parse("2.0.0").unwrap())
        .expect("d@2.0.0 should be present");
    let d3 = g
        .get_node("d", &semver::Version::parse("3.0.0").unwrap())
        .expect("d@3.0.0 should be present");
    assert_eq!(d2.version, "2.0.0");
    assert_eq!(d3.version, "3.0.0");
    // Each d variant must remember which parent pulled it in.
    assert_eq!(d2.required_by, vec!["b@1.0.0".to_string()]);
    assert_eq!(d3.required_by, vec!["c@1.0.0".to_string()]);
}

/// `node_count` is the total (name, version) pairs — not the unique
/// name count. With the diamond, the graph has 5 entries (a, b, c, d×2).
#[test]
fn diamond_node_count_is_5() {
    let g = diamond_graph();
    assert_eq!(g.node_count(), 5);
}

/// `analyze_npm_graph` must surface the diamond as a conflict chain
/// (it iterates *all* versions, not just the first).
#[test]
fn diamond_emits_conflict_chain() {
    use ven::intelligence::conflicts::analyze_npm_graph;
    let g = diamond_graph();
    let existing = std::collections::HashMap::new();
    let (chains, _) = analyze_npm_graph(&g, &existing);
    assert!(
        chains
            .iter()
            .any(|c| c.package == "d" && c.steps.iter().any(|s| s.contains("multiple versions"))),
        "expected diamond chain, got: {:?}",
        chains
    );
}
