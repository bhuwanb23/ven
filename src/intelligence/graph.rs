//! Normalized dependency graph types for the intelligence layer.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Which ecosystem this graph represents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeKind {
    NpmFamily, // node or bun (npm registry)
    Python,
    Go,
    Rust,
    Java,
    Deno,
    Ruby,
    Stub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelNode {
    pub name: String,
    pub version: String,
    pub depth: u32,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub engines_node: Option<String>,
    #[serde(default)]
    pub deprecated: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub required_by: Vec<String>,
    /// SRI-style integrity from upstream registry (npm `dist.integrity`),
    /// e.g. `sha512-...` or `sha256-...`. `None` for ecosystems that do not
    /// publish per-version checksums.
    #[serde(default)]
    pub integrity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntelEdge {
    pub from: String,
    pub to: String,
    pub constraint: String,
    #[serde(default)]
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum EdgeKind {
    #[default]
    Dependency,
    Peer,
    Dev,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelGraph {
    pub runtime_kind: RuntimeKind,
    pub runtime_version: String,
    /// Multi-version node map. Key is package name; value is a map of
    /// resolved version -> IntelNode.
    ///
    /// **Why multi-version?** A diamond dependency (a -> b@1, a -> c@1,
    /// b -> d@2, c -> d@3) resolves `d` to *two* versions. Storing only
    /// one loses the conflict and `ven add` would pick the wrong one
    /// silently. With this map, `analyze_npm_graph` iterates *every*
    /// version and `has_version_conflicts` is a single `O(n)` check.
    pub nodes: BTreeMap<String, BTreeMap<semver::Version, IntelNode>>,
    pub edges: Vec<IntelEdge>,
}

impl IntelGraph {
    /// Look up a specific (name, version) pair. Returns `None` if the
    /// name is absent or the version was not resolved.
    pub fn get_node(&self, name: &str, version: &semver::Version) -> Option<&IntelNode> {
        self.nodes.get(name).and_then(|m| m.get(version))
    }

    /// First IntelNode for `name` in insertion order. Useful for
    /// display paths that don't care which version.
    pub fn first_node(&self, name: &str) -> Option<&IntelNode> {
        self.nodes.get(name).and_then(|m| m.values().next())
    }

    /// All IntelNodes for `name`, one per resolved version.
    pub fn all_nodes(&self, name: &str) -> impl Iterator<Item = (&semver::Version, &IntelNode)> {
        self.nodes.get(name).into_iter().flat_map(|m| m.iter())
    }

    /// Total (name, version) pairs across the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.values().map(|m| m.len()).sum()
    }

    /// True iff any package name has more than one resolved version.
    pub fn has_version_conflicts(&self) -> bool {
        self.nodes.values().any(|m| m.len() > 1)
    }

    /// Names that resolve to more than one version (diamond deps).
    pub fn conflicted_names(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(_, m)| m.len() > 1)
            .map(|(n, _)| n.clone())
            .collect()
    }
}

/// One explainable conflict path (human-readable steps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictChain {
    pub steps: Vec<String>,
    pub package: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineIncompatibility {
    pub package: String,
    pub version: String,
    pub required_node: String,
    pub current_node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionOption {
    pub id: u32,
    pub label: String,
    pub action: ResolutionAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionAction {
    InstallVersion { package: String, version: String },
    Downgrade { package: String, version: String },
    Cancel,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub compatible: bool,
    pub graph: IntelGraph,
    pub conflict_chains: Vec<ConflictChain>,
    pub engine_incompatibilities: Vec<EngineIncompatibility>,
    pub suggestions: Vec<ResolutionOption>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckAddResult {
    pub stack_summary: String,
    pub compatible_versions: Vec<String>,
    pub incompatible_examples: Vec<(String, String)>,
    pub recommended: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl SimulationResult {
    pub fn stub_compatible(runtime_kind: RuntimeKind, runtime_version: &str, msg: &str) -> Self {
        Self {
            compatible: true,
            graph: IntelGraph {
                runtime_kind,
                runtime_version: runtime_version.to_string(),
                nodes: BTreeMap::new(),
                edges: Vec::new(),
            },
            conflict_chains: Vec::new(),
            engine_incompatibilities: Vec::new(),
            suggestions: Vec::new(),
            warnings: vec![msg.to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_node(name: &str, version: &str) -> IntelNode {
        IntelNode {
            name: name.into(),
            version: version.into(),
            depth: 0,
            dependencies: std::collections::HashMap::new(),
            engines_node: None,
            deprecated: None,
            license: None,
            size_bytes: None,
            required_by: vec![],
            integrity: None,
        }
    }

    fn mk_graph() -> IntelGraph {
        IntelGraph {
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            nodes: BTreeMap::new(),
            edges: vec![],
        }
    }

    /// A package that resolves to two versions (diamond dep) must be stored
    /// under both keys, and `has_version_conflicts` / `conflicted_names`
    /// must surface it.
    #[test]
    fn multi_version_node_storage() {
        let mut g = mk_graph();
        g.nodes.entry("d".into()).or_default().insert(
            semver::Version::parse("2.0.0").unwrap(),
            mk_node("d", "2.0.0"),
        );
        g.nodes.entry("d".into()).or_default().insert(
            semver::Version::parse("3.0.0").unwrap(),
            mk_node("d", "3.0.0"),
        );

        assert_eq!(g.node_count(), 2);
        assert!(g.has_version_conflicts());
        assert_eq!(g.conflicted_names(), vec!["d".to_string()]);
    }

    /// A package with one resolved version is *not* a conflict.
    #[test]
    fn single_version_is_not_a_conflict() {
        let mut g = mk_graph();
        g.nodes.entry("foo".into()).or_default().insert(
            semver::Version::parse("1.0.0").unwrap(),
            mk_node("foo", "1.0.0"),
        );
        assert!(!g.has_version_conflicts());
        assert!(g.conflicted_names().is_empty());
        assert_eq!(g.node_count(), 1);
    }

    /// `get_node` finds a specific (name, version) pair.
    #[test]
    fn get_node_finds_specific_version() {
        let mut g = mk_graph();
        g.nodes.entry("x".into()).or_default().insert(
            semver::Version::parse("1.0.0").unwrap(),
            mk_node("x", "1.0.0"),
        );
        g.nodes.entry("x".into()).or_default().insert(
            semver::Version::parse("2.0.0").unwrap(),
            mk_node("x", "2.0.0"),
        );

        assert!(g
            .get_node("x", &semver::Version::parse("1.0.0").unwrap())
            .is_some());
        assert!(g
            .get_node("x", &semver::Version::parse("2.0.0").unwrap())
            .is_some());
        assert!(g
            .get_node("x", &semver::Version::parse("9.9.9").unwrap())
            .is_none());
        assert!(g
            .get_node("y", &semver::Version::parse("1.0.0").unwrap())
            .is_none());
    }

    /// `first_node` returns *some* version (insertion order) and is
    /// independent of which version is "primary" — important for
    /// display code that doesn't care about the multi-version detail.
    #[test]
    fn first_node_returns_any_version() {
        let mut g = mk_graph();
        g.nodes.entry("z".into()).or_default().insert(
            semver::Version::parse("5.0.0").unwrap(),
            mk_node("z", "5.0.0"),
        );
        let first = g.first_node("z").unwrap();
        assert_eq!(first.version, "5.0.0");
    }
}
