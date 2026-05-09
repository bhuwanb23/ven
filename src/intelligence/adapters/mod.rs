//! Runtime adapters: npm family (Node/Bun) plus deterministic stubs for other ecosystems.

mod npm;

pub use npm::{
    find_highest_node_compatible_version, npm_check_add, resolve_version, NpmGraphBuilder,
};

use crate::core::config::VenConfig;
use crate::intelligence::conflicts::{analyze_npm_graph, engine_checks};
use crate::intelligence::graph::{
    CheckAddResult, IntelGraph, IntelNode, RuntimeKind, SimulationResult,
};
use crate::intelligence::suggestions::merge_suggestions;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Build a normalized dependency graph and run constraint checks (per ecosystem).
///
/// `?Send` because [`crate::core::npm_registry::NpmRegistry`] uses a non-`Send` SQLite cache.
#[async_trait(?Send)]
pub trait DependencyRuntimeAdapter {
    fn runtime_kind(&self) -> RuntimeKind;
    fn runtime_version(&self) -> &str;

    async fn simulate_add(
        &self,
        package: &str,
        version_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult>;

    async fn simulate_upgrade(
        &self,
        package: &str,
        target_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult>;

    async fn current_environment_graph(
        &self,
        manifest_packages: &HashMap<String, String>,
    ) -> Result<IntelGraph>;

    async fn check_add(
        &self,
        package: &str,
        version_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<CheckAddResult>;
}

/// npm registry semantics (Node + Bun).
pub struct NpmFamilyAdapter {
    kind: RuntimeKind,
    runtime_version: String,
}

impl NpmFamilyAdapter {
    pub fn new(runtime_version: String) -> Self {
        Self {
            kind: RuntimeKind::NpmFamily,
            runtime_version,
        }
    }

    async fn run_simulation(
        &self,
        package: &str,
        version_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        let mut builder = NpmGraphBuilder::new(self.runtime_version.clone(), self.kind.clone())?;
        let graph = builder
            .build(package, version_spec, existing_packages)
            .await?;
        let engine_incompatibilities = engine_checks(&graph);
        let (conflict_chains, suggestions) = analyze_npm_graph(&graph, existing_packages);
        let compatible = engine_incompatibilities.is_empty() && conflict_chains.is_empty();
        let mut result = SimulationResult {
            compatible,
            graph,
            conflict_chains,
            engine_incompatibilities,
            suggestions,
            warnings: Vec::new(),
        };
        merge_suggestions(&mut result);
        Ok(result)
    }
}

#[async_trait(?Send)]
impl DependencyRuntimeAdapter for NpmFamilyAdapter {
    fn runtime_kind(&self) -> RuntimeKind {
        self.kind.clone()
    }

    fn runtime_version(&self) -> &str {
        &self.runtime_version
    }

    async fn simulate_add(
        &self,
        package: &str,
        version_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        self.run_simulation(package, version_spec, existing_packages)
            .await
    }

    async fn simulate_upgrade(
        &self,
        package: &str,
        target_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        let mut pins = existing_packages.clone();
        pins.remove(package);
        self.run_simulation(package, target_spec, &pins).await
    }

    async fn current_environment_graph(
        &self,
        manifest_packages: &HashMap<String, String>,
    ) -> Result<IntelGraph> {
        let mut builder = NpmGraphBuilder::new(self.runtime_version.clone(), self.kind.clone())?;
        let graph = builder
            .build_workspace(manifest_packages, manifest_packages)
            .await?;
        Ok(graph)
    }

    async fn check_add(
        &self,
        package: &str,
        version_spec: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<CheckAddResult> {
        npm_check_add(
            package,
            version_spec,
            &self.runtime_version,
            self.kind.clone(),
            existing_packages,
        )
        .await
    }
}

/// Deterministic placeholder: records limitations, does not block installs.
pub struct GenericStubAdapter {
    kind: RuntimeKind,
    runtime_version: String,
}

impl GenericStubAdapter {
    pub fn new(kind: RuntimeKind, runtime_version: String) -> Self {
        Self {
            kind,
            runtime_version,
        }
    }
}

#[async_trait(?Send)]
impl DependencyRuntimeAdapter for GenericStubAdapter {
    fn runtime_kind(&self) -> RuntimeKind {
        self.kind.clone()
    }

    fn runtime_version(&self) -> &str {
        &self.runtime_version
    }

    async fn simulate_add(
        &self,
        _package: &str,
        _version_spec: &str,
        _existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        Ok(SimulationResult::stub_compatible(
            self.kind.clone(),
            &self.runtime_version,
            "Best-effort stub: full npm-style simulation not wired for this runtime; install may still proceed.",
        ))
    }

    async fn simulate_upgrade(
        &self,
        _package: &str,
        _target_spec: &str,
        _existing_packages: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        Ok(SimulationResult::stub_compatible(
            self.kind.clone(),
            &self.runtime_version,
            "Stub upgrade simulation — verify with ecosystem-native tools.",
        ))
    }

    async fn current_environment_graph(
        &self,
        manifest_packages: &HashMap<String, String>,
    ) -> Result<IntelGraph> {
        let mut nodes = HashMap::new();
        for (name, pinned) in manifest_packages {
            nodes.insert(
                name.clone(),
                IntelNode {
                    name: name.clone(),
                    version: pinned.clone(),
                    depth: 0,
                    dependencies: HashMap::new(),
                    engines_node: None,
                    deprecated: None,
                    license: None,
                    size_bytes: None,
                    required_by: Vec::new(),
                },
            );
        }
        Ok(IntelGraph {
            runtime_kind: self.kind.clone(),
            runtime_version: self.runtime_version.clone(),
            nodes,
            edges: Vec::new(),
        })
    }

    async fn check_add(
        &self,
        package: &str,
        version_spec: &str,
        _existing: &HashMap<String, String>,
    ) -> Result<CheckAddResult> {
        Ok(CheckAddResult {
            stack_summary: format!("{:?} {}", self.kind, self.runtime_version),
            compatible_versions: vec![version_spec.to_string()],
            incompatible_examples: Vec::new(),
            recommended: Some(version_spec.to_string()),
            warnings: vec![format!(
                "Stub check-add for `{}` on {:?} — confirm with ecosystem documentation.",
                package, self.kind
            )],
        })
    }
}

/// Select adapter from `ven.toml` primary runtime (same precedence as `ven add`).
pub fn adapter_from_ven_config(cfg: &VenConfig) -> Box<dyn DependencyRuntimeAdapter> {
    let r = &cfg.runtime;
    if !r.python.is_empty() && r.node.is_empty() && r.bun.is_empty() {
        return Box::new(GenericStubAdapter::new(
            RuntimeKind::Python,
            r.python.clone(),
        ));
    }
    if !r.go.is_empty()
        && r.node.is_empty()
        && r.python.is_empty()
        && r.ruby.is_empty()
        && r.bun.is_empty()
    {
        return Box::new(GenericStubAdapter::new(RuntimeKind::Go, r.go.clone()));
    }
    if !r.rust.is_empty()
        && r.node.is_empty()
        && r.python.is_empty()
        && r.go.is_empty()
        && r.ruby.is_empty()
        && r.bun.is_empty()
    {
        return Box::new(GenericStubAdapter::new(RuntimeKind::Rust, r.rust.clone()));
    }
    if !r.java.is_empty()
        && r.node.is_empty()
        && r.python.is_empty()
        && r.go.is_empty()
        && r.rust.is_empty()
        && r.ruby.is_empty()
        && r.bun.is_empty()
    {
        return Box::new(GenericStubAdapter::new(RuntimeKind::Java, r.java.clone()));
    }
    if !r.deno.is_empty()
        && r.node.is_empty()
        && r.python.is_empty()
        && r.go.is_empty()
        && r.rust.is_empty()
        && r.java.is_empty()
        && r.ruby.is_empty()
        && r.bun.is_empty()
    {
        return Box::new(GenericStubAdapter::new(RuntimeKind::Deno, r.deno.clone()));
    }
    if !r.bun.is_empty()
        && r.node.is_empty()
        && r.python.is_empty()
        && r.go.is_empty()
        && r.rust.is_empty()
        && r.java.is_empty()
        && r.deno.is_empty()
        && r.ruby.is_empty()
    {
        return Box::new(NpmFamilyAdapter::new(r.bun.clone()));
    }
    if !r.ruby.is_empty()
        && r.node.is_empty()
        && r.python.is_empty()
        && r.go.is_empty()
        && r.rust.is_empty()
        && r.java.is_empty()
        && r.deno.is_empty()
    {
        return Box::new(GenericStubAdapter::new(RuntimeKind::Ruby, r.ruby.clone()));
    }
    if !r.node.is_empty() {
        return Box::new(NpmFamilyAdapter::new(r.node.clone()));
    }
    Box::new(GenericStubAdapter::new(RuntimeKind::Stub, String::new()))
}
