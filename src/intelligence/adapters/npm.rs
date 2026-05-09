//! npm registry graph builder (Node + Bun).

use crate::core::npm_registry::{NpmRegistry, PackageMetadata};
use crate::intelligence::conflicts::node_engine_satisfies;
use crate::intelligence::graph::{
    CheckAddResult, EdgeKind, IntelEdge, IntelGraph, IntelNode, RuntimeKind,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct NpmGraphBuilder {
    pub nodes: HashMap<String, IntelNode>,
    pub edges: Vec<IntelEdge>,
    registry: NpmRegistry,
    runtime_version: String,
    runtime_kind: RuntimeKind,
}

impl NpmGraphBuilder {
    pub fn new(runtime_version: String, runtime_kind: RuntimeKind) -> Result<Self> {
        Ok(Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            registry: NpmRegistry::new()?,
            runtime_version,
            runtime_kind,
        })
    }

    pub async fn build(
        &mut self,
        root_package: &str,
        root_version: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<IntelGraph> {
        self.build_package(root_package, root_version, existing_packages)
            .await?;
        self.add_peer_edges().await?;

        Ok(IntelGraph {
            runtime_kind: self.runtime_kind.clone(),
            runtime_version: self.runtime_version.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        })
    }

    pub async fn build_workspace(
        &mut self,
        root_packages: &HashMap<String, String>,
        existing_packages: &HashMap<String, String>,
    ) -> Result<IntelGraph> {
        let mut names: Vec<_> = root_packages.keys().collect();
        names.sort();
        for name in names {
            let version = root_packages.get(name).unwrap_or(&"latest".to_string());
            if self.nodes.contains_key(name) {
                continue;
            }
            self.build_package(name, version, existing_packages).await?;
        }
        self.add_peer_edges().await?;

        Ok(IntelGraph {
            runtime_kind: self.runtime_kind.clone(),
            runtime_version: self.runtime_version.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        })
    }

    async fn build_package(
        &mut self,
        root_package: &str,
        root_version: &str,
        existing_packages: &HashMap<String, String>,
    ) -> Result<()> {
        let metadata = self.registry.fetch_package_metadata(root_package).await?;
        let resolved_version = resolve_version(&metadata, root_version)?;
        self.add_node(root_package, &resolved_version, &metadata, 0, None)?;
        Box::pin(self.fetch_deps(root_package, &resolved_version, 0)).await?;
        let _ = existing_packages;
        Ok(())
    }

    async fn add_peer_edges(&mut self) -> Result<()> {
        let snapshot: Vec<(String, String)> = self
            .nodes
            .iter()
            .map(|(k, n)| (k.clone(), n.version.clone()))
            .collect();

        for (pkg_name, pkg_version) in snapshot {
            let meta = match self.registry.fetch_package_metadata(&pkg_name).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            let Some(vm) = meta.versions.get(&pkg_version) else {
                continue;
            };
            let peers = vm.peer_dependencies.clone().unwrap_or_default();
            let from = format!("{}@{}", pkg_name, pkg_version);
            for (peer_name, constraint) in peers {
                let resolved_ver = self
                    .nodes
                    .get(&peer_name)
                    .map(|n| n.version.clone())
                    .unwrap_or_else(|| "?".into());
                let to = format!("{}@{}", peer_name, resolved_ver);
                self.edges.push(IntelEdge {
                    from: from.clone(),
                    to,
                    constraint,
                    kind: EdgeKind::Peer,
                });
            }
        }
        Ok(())
    }

    async fn fetch_deps(&mut self, package: &str, version: &str, depth: u32) -> Result<()> {
        if depth > 20 {
            return Err(anyhow!("Dependency tree too deep (>20)"));
        }
        let version_meta = match self.registry.fetch_version_metadata(package, version).await {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };
        let dependencies = version_meta.dependencies.unwrap_or_default();

        for (dep_name, dep_constraint) in &dependencies {
            let dep_metadata = match self.registry.fetch_package_metadata(dep_name).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            let dep_version = match resolve_version(&dep_metadata, dep_constraint) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let edge_from = format!("{}@{}", package, version);
            let edge_to = format!("{}@{}", dep_name, dep_version);
            self.edges.push(IntelEdge {
                from: edge_from.clone(),
                to: edge_to.clone(),
                constraint: dep_constraint.clone(),
                kind: EdgeKind::Dependency,
            });

            if self.nodes.contains_key(dep_name) {
                if let Some(node) = self.nodes.get_mut(dep_name) {
                    node.required_by.push(edge_from);
                }
            } else {
                self.add_node(
                    dep_name,
                    &dep_version,
                    &dep_metadata,
                    depth + 1,
                    Some(&edge_from),
                )?;
                Box::pin(self.fetch_deps(dep_name, &dep_version, depth + 1)).await?;
            }
        }
        Ok(())
    }

    fn add_node(
        &mut self,
        name: &str,
        version: &str,
        metadata: &PackageMetadata,
        depth: u32,
        required_by: Option<&str>,
    ) -> Result<()> {
        let version_meta = metadata.versions.get(version);
        let dependencies = version_meta
            .and_then(|v| v.dependencies.clone())
            .unwrap_or_default();
        let engines_node = version_meta
            .and_then(|v| v.engines.clone())
            .and_then(|e| e.node);
        let deprecated = version_meta.and_then(|v| v.deprecated.clone());
        let license = version_meta.and_then(|v| v.license.clone());
        let size_bytes = version_meta
            .and_then(|v| v.dist.clone())
            .and_then(|d| d.unpacked_size);

        let node = IntelNode {
            name: name.to_string(),
            version: version.to_string(),
            depth,
            dependencies,
            engines_node,
            deprecated,
            license,
            size_bytes,
            required_by: required_by.map(|s| vec![s.to_string()]).unwrap_or_default(),
        };
        self.nodes.insert(name.to_string(), node);
        Ok(())
    }
}

pub fn resolve_version(metadata: &PackageMetadata, constraint: &str) -> Result<String> {
    if constraint == "latest" {
        return metadata
            .dist_tags
            .get("latest")
            .cloned()
            .ok_or_else(|| anyhow!("No 'latest' tag"));
    }
    if constraint == "lts" {
        let mut versions: Vec<semver::Version> = metadata
            .versions
            .keys()
            .filter_map(|v| semver::Version::parse(v).ok())
            .filter(|v| v.major % 2 == 0)
            .collect();
        versions.sort_by(|a, b| b.cmp(a));
        return versions
            .first()
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow!("No LTS versions"));
    }
    match semver::VersionReq::parse(constraint) {
        Ok(req) => {
            let mut versions: Vec<semver::Version> = metadata
                .versions
                .keys()
                .filter_map(|v| semver::Version::parse(v).ok())
                .filter(|v| req.matches(v))
                .collect();
            versions.sort_by(|a, b| b.cmp(a));
            versions
                .first()
                .map(|v| v.to_string())
                .ok_or_else(|| anyhow!("No version matches '{}'", constraint))
        }
        Err(_) => semver::Version::parse(constraint)
            .map(|v| v.to_string())
            .or_else(|_| {
                if let Ok(major) = constraint.parse::<u64>() {
                    let mut versions: Vec<semver::Version> = metadata
                        .versions
                        .keys()
                        .filter_map(|v| semver::Version::parse(v).ok())
                        .filter(|v| v.major == major)
                        .collect();
                    versions.sort_by(|a, b| b.cmp(a));
                    return versions
                        .first()
                        .map(|v| v.to_string())
                        .ok_or_else(|| anyhow!("No version {}.*", major));
                }
                Err(anyhow!("Invalid constraint {}", constraint))
            }),
    }
}

/// Highest published version whose `engines.node` satisfies `node_version` (npm upgrade preview).
pub fn find_highest_node_compatible_version(
    metadata: &PackageMetadata,
    node_version: &str,
) -> Option<String> {
    if let Some(latest) = metadata.dist_tags.get("latest") {
        if version_node_engine_compatible(metadata, latest, node_version) {
            return Some(latest.clone());
        }
    }
    let mut versions: Vec<semver::Version> = metadata
        .versions
        .keys()
        .filter_map(|v| semver::Version::parse(v).ok())
        .collect();
    versions.sort_by(|a, b| b.cmp(a));
    versions
        .into_iter()
        .find(|v| version_node_engine_compatible(metadata, &v.to_string(), node_version))
        .map(|v| v.to_string())
}

fn version_node_engine_compatible(
    metadata: &PackageMetadata,
    pkg_ver: &str,
    node_ver: &str,
) -> bool {
    let Some(vm) = metadata.versions.get(pkg_ver) else {
        return false;
    };
    let Some(ref eng) = vm.engines else {
        return true;
    };
    let Some(ref node_req) = eng.node else {
        return true;
    };
    node_engine_satisfies(node_ver, node_req)
}

/// Non-mutating check: semver candidates, engine samples, plus full simulation warnings.
pub async fn npm_check_add(
    package: &str,
    version_spec: &str,
    node_version: &str,
    runtime_kind: RuntimeKind,
    existing: &HashMap<String, String>,
) -> Result<CheckAddResult> {
    use crate::intelligence::conflicts::{analyze_npm_graph, engine_checks};

    let registry = NpmRegistry::new()?;
    let meta = registry.fetch_package_metadata(package).await?;
    let req_parsed = semver::VersionReq::parse(version_spec).ok();
    let mut compatible_versions = Vec::new();
    let mut incompatible_examples: Vec<(String, String)> = Vec::new();
    let mut versions: Vec<semver::Version> = meta
        .versions
        .keys()
        .filter_map(|v| semver::Version::parse(v).ok())
        .collect();
    versions.sort_by(|a, b| b.cmp(a));
    for v in versions.iter().take(200) {
        if !matches!(version_spec, "latest" | "lts") {
            if let Some(ref r) = req_parsed {
                if !r.matches(v) {
                    continue;
                }
            }
        }
        let s = v.to_string();
        if version_node_engine_compatible(&meta, &s, node_version) {
            if compatible_versions.len() < 15 {
                compatible_versions.push(s);
            }
        } else if incompatible_examples.len() < 6 {
            let req_text = meta
                .versions
                .get(&s)
                .and_then(|vm| vm.engines.as_ref())
                .and_then(|e| e.node.clone())
                .unwrap_or_else(|| "?".into());
            incompatible_examples.push((s, format!("needs Node {}", req_text)));
        }
    }
    let recommended = resolve_version(&meta, version_spec).ok();
    let mut builder = NpmGraphBuilder::new(node_version.to_string(), runtime_kind.clone())?;
    let graph = builder.build(package, version_spec, existing).await?;
    let engine_inc = engine_checks(&graph);
    let (chains, _) = analyze_npm_graph(&graph, existing);
    let mut warnings = Vec::new();
    if !chains.is_empty() {
        warnings.push(format!(
            "{} peer/pin conflict(s) in simulated graph",
            chains.len()
        ));
    }
    if !engine_inc.is_empty() {
        warnings.push(format!(
            "{} engine constraint(s) failed in simulated graph",
            engine_inc.len()
        ));
    }
    Ok(CheckAddResult {
        stack_summary: format!("{:?} {}", runtime_kind, node_version),
        compatible_versions,
        incompatible_examples,
        recommended,
        warnings,
    })
}
