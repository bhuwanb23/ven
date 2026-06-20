//! Orchestration entry points for CLI and tooling.

use crate::core::config::VenConfig;
use crate::core::npm_registry::NpmRegistry;
use crate::intelligence::adapters::{
    adapter_from_ven_config, find_highest_node_compatible_version,
};
use crate::intelligence::graph::{CheckAddResult, IntelGraph, SimulationResult};
use crate::intelligence::store::IntelligenceStore;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Dependency intelligence facade (pre-install simulation, snapshots, queries).
pub struct DependencyIntelligenceService;

impl DependencyIntelligenceService {
    pub fn project_key(cwd: &Path) -> String {
        cwd.canonicalize()
            .unwrap_or_else(|_| cwd.to_path_buf())
            .to_string_lossy()
            .into_owned()
    }

    pub fn block_on<F, Fut, T>(make_fut: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        crate::core::block_on_async(make_fut())?
    }

    pub fn simulate_add(
        cfg: &VenConfig,
        package: &str,
        version_spec: &str,
        existing: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        let adapter = adapter_from_ven_config(cfg);
        let ex = existing.clone();
        let pkg = package.to_string();
        let vs = version_spec.to_string();
        Self::block_on(move || async move { adapter.simulate_add(&pkg, &vs, &ex).await })
    }

    pub fn simulate_upgrade(
        cfg: &VenConfig,
        package: &str,
        target_spec: &str,
        existing: &HashMap<String, String>,
    ) -> Result<SimulationResult> {
        let adapter = adapter_from_ven_config(cfg);
        let ex = existing.clone();
        let pkg = package.to_string();
        let ts = target_spec.to_string();
        Self::block_on(move || async move { adapter.simulate_upgrade(&pkg, &ts, &ex).await })
    }

    pub fn check_add(
        cfg: &VenConfig,
        package: &str,
        version_spec: &str,
        existing: &HashMap<String, String>,
    ) -> Result<CheckAddResult> {
        let adapter = adapter_from_ven_config(cfg);
        let ex = existing.clone();
        let pkg = package.to_string();
        let vs = version_spec.to_string();
        Self::block_on(move || async move { adapter.check_add(&pkg, &vs, &ex).await })
    }

    pub fn environment_graph(
        cfg: &VenConfig,
        manifest_packages: &HashMap<String, String>,
    ) -> Result<IntelGraph> {
        let adapter = adapter_from_ven_config(cfg);
        let m = manifest_packages.clone();
        Self::block_on(move || async move { adapter.current_environment_graph(&m).await })
    }

    /// Highest registry version compatible with the configured Node runtime (npm/bun projects).
    pub fn npm_latest_compatible(package: &str, node_version: &str) -> Result<Option<String>> {
        Self::block_on(|| async move {
            let reg = NpmRegistry::new()?;
            let meta = reg.fetch_package_metadata(package).await?;
            Ok(find_highest_node_compatible_version(&meta, node_version))
        })
    }

    pub fn persist_snapshot(project_key: &str, result: &SimulationResult) -> Result<()> {
        let store = IntelligenceStore::open()?;
        store.save(project_key, result)
    }

    pub fn load_snapshot(project_key: &str) -> Result<Option<SimulationResult>> {
        let store = IntelligenceStore::open()?;
        store.load(project_key)
    }

    /// Dependents from `package-lock.json` (npm); single entry point for `ven remove` analysis.
    pub fn list_npm_lockfile_dependents(package: &str) -> Result<Vec<(String, String)>> {
        crate::core::packages::find_dependents(package)
    }
}
