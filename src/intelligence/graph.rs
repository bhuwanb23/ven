//! Normalized dependency graph types for the intelligence layer.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub nodes: HashMap<String, IntelNode>,
    pub edges: Vec<IntelEdge>,
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
                nodes: HashMap::new(),
                edges: Vec::new(),
            },
            conflict_chains: Vec::new(),
            engine_incompatibilities: Vec::new(),
            suggestions: Vec::new(),
            warnings: vec![msg.to_string()],
        }
    }
}
