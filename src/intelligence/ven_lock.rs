//! `ven.lock` format: pinned graph for reproducible installs and `ven sync` validation.

use crate::intelligence::conflicts::version_satisfies_constraint;
use crate::intelligence::graph::{EdgeKind, IntelEdge, IntelGraph, IntelNode, RuntimeKind};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub const LOCK_FORMAT_VERSION: u32 = 1;
pub const DEFAULT_ECOSYSTEM: &str = "npm";

/// One pinned package in `ven.lock`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenLockPackage {
    pub version: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// One dependency edge in `ven.lock`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VenLockEdge {
    pub from: String,
    pub to: String,
    pub constraint: String,
    #[serde(default)]
    pub kind: EdgeKind,
}

/// Full lockfile document (JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenLockFile {
    pub lock_format_version: u32,
    pub ecosystem: String,
    pub runtime_kind: RuntimeKind,
    pub runtime_version: String,
    pub roots: Vec<String>,
    pub packages: HashMap<String, VenLockPackage>,
    pub edges: Vec<VenLockEdge>,
    /// SHA-256 hex of canonical payload (excluding this field when computing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

impl VenLockFile {
    /// Read and verify `content_hash` when present.
    pub fn read_path(path: &Path) -> Result<Self> {
        let raw =
            fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        let lock: VenLockFile =
            serde_json::from_str(&raw).with_context(|| format!("Invalid ven.lock at {:?}", path))?;
        let recomputed = {
            let mut tmp = lock.clone();
            tmp.content_hash = None;
            compute_lock_content_hash(&tmp)?
        };
        if let Some(ref h) = lock.content_hash {
            if h != &recomputed {
                return Err(anyhow!(
                    "ven.lock content_hash mismatch (corrupted or hand-edited). File says {}, recomputed {}",
                    h,
                    recomputed
                ));
            }
        }
        Ok(lock)
    }

    /// Merge simulation graphs for each root in `cfg.packages` (npm family only).
    pub fn from_merged_simulations(
        runtime_kind: RuntimeKind,
        runtime_version: String,
        roots_order: &[String],
        graphs: &[IntelGraph],
    ) -> Result<Self> {
        if graphs.is_empty() {
            return Err(anyhow!("No graphs to merge"));
        }
        let mut merged = IntelGraph {
            runtime_kind: runtime_kind.clone(),
            runtime_version: runtime_version.clone(),
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        for g in graphs {
            merge_intel_graph(&mut merged, g)?;
        }

        let mut roots: Vec<String> = roots_order.to_vec();
        roots.sort();
        roots.dedup();

        let mut packages = HashMap::new();
        for (name, node) in &merged.nodes {
            packages.insert(
                name.clone(),
                VenLockPackage {
                    version: node.version.clone(),
                    metadata: None,
                },
            );
        }

        let edges: Vec<VenLockEdge> = merged
            .edges
            .iter()
            .map(|e| VenLockEdge {
                from: e.from.clone(),
                to: e.to.clone(),
                constraint: e.constraint.clone(),
                kind: e.kind.clone(),
            })
            .collect();

        let mut lock = VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: DEFAULT_ECOSYSTEM.to_string(),
            runtime_kind,
            runtime_version,
            roots,
            packages,
            edges,
            content_hash: None,
        };
        lock.content_hash = Some(compute_lock_content_hash(&lock)?);
        Ok(lock)
    }

    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn write_path(&self, path: &Path) -> Result<()> {
        let s = self.to_json_pretty()?;
        fs::write(path, s).with_context(|| format!("Failed to write {:?}", path))?;
        Ok(())
    }
}

/// Rebuild an [`IntelGraph`] from a validated lock (for peer / pin analysis).
pub fn lock_to_intel_graph(lock: &VenLockFile) -> IntelGraph {
    let nodes: HashMap<String, IntelNode> = lock
        .packages
        .iter()
        .map(|(name, p)| {
            (
                name.clone(),
                IntelNode {
                    name: name.clone(),
                    version: p.version.clone(),
                    depth: 0,
                    dependencies: HashMap::new(),
                    engines_node: None,
                    deprecated: None,
                    license: None,
                    size_bytes: None,
                    required_by: Vec::new(),
                },
            )
        })
        .collect();
    let edges: Vec<IntelEdge> = lock
        .edges
        .iter()
        .map(|e| IntelEdge {
            from: e.from.clone(),
            to: e.to.clone(),
            constraint: e.constraint.clone(),
            kind: e.kind.clone(),
        })
        .collect();
    IntelGraph {
        runtime_kind: lock.runtime_kind.clone(),
        runtime_version: lock.runtime_version.clone(),
        nodes,
        edges,
    }
}

fn merge_intel_graph(dst: &mut IntelGraph, src: &IntelGraph) -> Result<()> {
    for (name, node) in &src.nodes {
        if let Some(existing) = dst.nodes.get(name) {
            if existing.version != node.version {
                return Err(anyhow!(
                    "Cannot merge lock graph: package `{}` appears as {} and {}",
                    name,
                    existing.version,
                    node.version
                ));
            }
        } else {
            dst.nodes.insert(name.clone(), node.clone());
        }
    }
    for e in &src.edges {
        if !dst.edges.iter().any(|x| x == e) {
            dst.edges.push(e.clone());
        }
    }
    Ok(())
}

/// Hash canonical JSON of the lock **without** `content_hash` set.
pub fn compute_lock_content_hash(lock: &VenLockFile) -> Result<String> {
    let mut tmp = lock.clone();
    tmp.content_hash = None;
    let json = serde_json::to_string(&tmp)?;
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// Structural + semver consistency checks.
pub fn validate_lock_graph(lock: &VenLockFile) -> Result<()> {
    let mut errors = Vec::new();

    if lock.lock_format_version != LOCK_FORMAT_VERSION {
        errors.push(format!(
            "unsupported lock_format_version {} (expected {})",
            lock.lock_format_version, LOCK_FORMAT_VERSION
        ));
    }

    if lock.packages.is_empty() {
        errors.push("lock has no packages".into());
    }

    for root in &lock.roots {
        if !lock.packages.contains_key(root) {
            errors.push(format!("root `{}` missing from packages map", root));
        }
    }

    let mut referenced: HashSet<String> = lock.roots.iter().cloned().collect();

    for (i, edge) in lock.edges.iter().enumerate() {
        let Some((from_pkg, from_ver)) = edge.from.rsplit_once('@') else {
            errors.push(format!("edge[{}]: invalid `from` (expected name@version): {}", i, edge.from));
            continue;
        };
        let Some((to_pkg, to_ver)) = edge.to.rsplit_once('@') else {
            errors.push(format!("edge[{}]: invalid `to` (expected name@version): {}", i, edge.to));
            continue;
        };

        referenced.insert(from_pkg.to_string());
        referenced.insert(to_pkg.to_string());

        match lock.packages.get(from_pkg) {
            Some(p) if p.version == from_ver => {}
            Some(p) => errors.push(format!(
                "edge[{}]: `from` {}@{} does not match locked version {}",
                i, from_pkg, from_ver, p.version
            )),
            None => errors.push(format!(
                "edge[{}]: `from` package `{}` not in packages map",
                i, from_pkg
            )),
        }

        match lock.packages.get(to_pkg) {
            Some(p) if p.version == to_ver => {}
            Some(p) => errors.push(format!(
                "edge[{}]: `to` {}@{} does not match locked version {}",
                i, to_pkg, to_ver, p.version
            )),
            None => errors.push(format!(
                "edge[{}]: `to` package `{}` not in packages map",
                i, to_pkg
            )),
        }

        if matches!(edge.kind, EdgeKind::Dependency | EdgeKind::Peer) {
            if !version_satisfies_constraint(to_ver, &edge.constraint) {
                errors.push(format!(
                    "edge[{}]: `{}@{}` does not satisfy constraint `{}` ({:?})",
                    i, to_pkg, to_ver, edge.constraint, edge.kind
                ));
            }
        }
    }

    for name in lock.packages.keys() {
        if !referenced.contains(name) {
            errors.push(format!(
                "package `{}` is not listed in roots and does not appear on any edge endpoint",
                name
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("ven.lock validation failed:\n{}", errors.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_trivial_lock() {
        let lock = VenLockFile {
            lock_format_version: 1,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["left-pad".into()],
            packages: HashMap::from([(
                "left-pad".into(),
                VenLockPackage {
                    version: "1.3.0".into(),
                    metadata: None,
                },
            )]),
            edges: vec![],
            content_hash: None,
        };
        validate_lock_graph(&lock).unwrap();
    }

    #[test]
    fn hash_stable_without_content_hash_field() {
        let lock = VenLockFile {
            lock_format_version: 1,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["a".into()],
            packages: HashMap::from([(
                "a".into(),
                VenLockPackage {
                    version: "1.0.0".into(),
                    metadata: None,
                },
            )]),
            edges: vec![],
            content_hash: None,
        };
        let h1 = compute_lock_content_hash(&lock).unwrap();
        let h2 = compute_lock_content_hash(&lock).unwrap();
        assert_eq!(h1, h2);
    }
}
