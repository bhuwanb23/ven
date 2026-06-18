//! `ven.lock` format: pinned graph for reproducible installs and `ven sync` validation.

use crate::intelligence::conflicts::version_satisfies_constraint;
use crate::intelligence::graph::{EdgeKind, IntelEdge, IntelGraph, IntelNode, RuntimeKind};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Current writer format. v2 introduced the per-package `integrity` field.
/// Reads of v1 lockfiles still succeed (integrity is `None`); on first
/// `ven sync` of a v1 lock we print a hint to regenerate.
pub const LOCK_FORMAT_VERSION: u32 = 2;
pub const MIN_SUPPORTED_LOCK_FORMAT_VERSION: u32 = 1;
pub const DEFAULT_ECOSYSTEM: &str = "npm";

/// One pinned package in `ven.lock`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenLockPackage {
    pub version: String,
    /// SRI-style integrity (e.g. `sha512-...` or `sha256-...`), copied from
    /// the upstream registry. npm-family only in v2; `None` for everything else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Validate an integrity string is in SRI form `<algo>-<base64>`.
/// Accepts `sha256`, `sha384`, `sha512` (npm uses sha512 today).
pub fn integrity_format_valid(s: &str) -> bool {
    let Some((algo, payload)) = s.split_once('-') else {
        return false;
    };
    if !matches!(algo, "sha256" | "sha384" | "sha512") {
        return false;
    }
    !payload.is_empty()
        && payload
            .bytes()
            .all(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'='))
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
    /// Read and verify `content_hash`.
    ///
    /// **v2 lockfiles MUST carry a `content_hash`**. A missing hash on a v2
    /// file is treated as tampering: an attacker who can edit the lockfile
    /// can also strip the hash, so accepting an unverified v2 file defeats
    /// the integrity guarantee. v1 lockfiles (the legacy format) are still
    /// read successfully — they predate the hash field — and the caller is
    /// expected to upgrade them via `ven lock`.
    pub fn read_path(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        let lock: VenLockFile = serde_json::from_str(&raw)
            .with_context(|| format!("Invalid ven.lock at {:?}", path))?;
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
        } else if lock.lock_format_version >= LOCK_FORMAT_VERSION {
            return Err(anyhow!(
                "ven.lock v{} is missing content_hash (required for v{}). Re-run `ven lock` to regenerate.",
                lock.lock_format_version,
                LOCK_FORMAT_VERSION
            ));
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
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        };
        for g in graphs {
            merge_intel_graph(&mut merged, g)?;
        }

        let mut roots: Vec<String> = roots_order.to_vec();
        roots.sort();
        roots.dedup();

        let mut packages = HashMap::new();
        // The lockfile's `packages` map is keyed by "name@version" so a
        // diamond dep yields *two* entries (`a@1.0.0` and `a@2.0.0`) rather
        // than clobbering. `validate_lock_graph` then checks the edges
        // point at the correct `name@version` key.
        for (name, versions) in &merged.nodes {
            for (_, node) in versions {
                let key = format!("{}@{}", name, node.version);
                packages.insert(
                    key,
                    VenLockPackage {
                        version: node.version.clone(),
                        integrity: node.integrity.clone(),
                        metadata: None,
                    },
                );
            }
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
        let tmp = path.with_extension("lock.tmp");
        fs::write(&tmp, s).with_context(|| format!("Failed to write {:?}", tmp))?;
        fs::rename(&tmp, path)
            .with_context(|| format!("Failed to atomically replace {:?}", path))?;
        Ok(())
    }
}

/// Rebuild an [`IntelGraph`] from a validated lock (for peer / pin analysis).
pub fn lock_to_intel_graph(lock: &VenLockFile) -> IntelGraph {
    // The lockfile's `packages` map is keyed by "name@version" (the merge
    // step in `from_merged_simulations` produces these composite keys so
    // diamond deps get *two* entries). We split each key back into
    // (name, version) and rebuild the multi-version IntelGraph.nodes.
    let mut nodes: BTreeMap<String, BTreeMap<semver::Version, IntelNode>> = BTreeMap::new();
    for (key, p) in &lock.packages {
        let (name, version) = match key.rsplit_once('@') {
            Some((n, v)) => (n.to_string(), v.to_string()),
            None => (key.clone(), p.version.clone()),
        };
        let v = semver::Version::parse(&version).unwrap_or_else(|_| {
            let mut fallback = semver::Version::new(0, 0, 0);
            fallback.pre =
                semver::Prerelease::new(&format!("ven-{}", version.replace('.', "_"))).unwrap();
            fallback
        });
        nodes.entry(name.clone()).or_default().insert(
            v,
            IntelNode {
                name,
                version,
                depth: 0,
                dependencies: HashMap::new(),
                engines_node: None,
                deprecated: None,
                license: None,
                size_bytes: None,
                required_by: Vec::new(),
                integrity: p.integrity.clone(),
            },
        );
    }
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
    for (name, versions) in &src.nodes {
        for (v, node) in versions {
            let entry = dst.nodes.entry(name.clone()).or_default();
            if let Some(existing) = entry.get(v) {
                if existing.version != node.version {
                    return Err(anyhow!(
                        "Cannot merge lock graph: package `{}` has version drift inside the same semver key",
                        name
                    ));
                }
            } else {
                entry.insert(v.clone(), node.clone());
            }
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
///
/// Important: `VenLockFile::packages` is a `HashMap` whose iteration order is
/// non-deterministic (Rust randomises the hasher per-process). Serialising it
/// directly with `to_string` would therefore yield a different byte sequence
/// across runs — and `ven sync --check` would always reject the file written
/// minutes earlier by `ven lock`.
///
/// We round-trip through `serde_json::Value` first: its `Map<String, Value>`
/// is a `BTreeMap` (sorted alphabetically), so the resulting JSON is stable
/// regardless of HashMap insertion order.
pub fn compute_lock_content_hash(lock: &VenLockFile) -> Result<String> {
    let mut tmp = lock.clone();
    tmp.content_hash = None;
    let value = serde_json::to_value(&tmp)?;
    let canonical = serde_json::to_string(&value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// True when `lock` was written by a writer older than this binary.
/// Read paths still succeed; callers should suggest `ven lock` to upgrade.
pub fn lock_needs_upgrade(lock: &VenLockFile) -> bool {
    lock.lock_format_version < LOCK_FORMAT_VERSION
}

/// Structural + semver consistency checks.
pub fn validate_lock_graph(lock: &VenLockFile) -> Result<()> {
    let mut errors = Vec::new();

    if lock.lock_format_version > LOCK_FORMAT_VERSION
        || lock.lock_format_version < MIN_SUPPORTED_LOCK_FORMAT_VERSION
    {
        errors.push(format!(
            "unsupported lock_format_version {} (this binary supports {}..={})",
            lock.lock_format_version, MIN_SUPPORTED_LOCK_FORMAT_VERSION, LOCK_FORMAT_VERSION
        ));
    }

    for (name, pkg) in &lock.packages {
        if let Some(ref s) = pkg.integrity {
            if !integrity_format_valid(s) {
                errors.push(format!(
                    "package `{}`@{}: invalid integrity `{}` (expected `<sha256|sha384|sha512>-<base64>`)",
                    name, pkg.version, s
                ));
            }
        }
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
            errors.push(format!(
                "edge[{}]: invalid `from` (expected name@version): {}",
                i, edge.from
            ));
            continue;
        };
        let Some((to_pkg, to_ver)) = edge.to.rsplit_once('@') else {
            errors.push(format!(
                "edge[{}]: invalid `to` (expected name@version): {}",
                i, edge.to
            ));
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
        Err(anyhow!(
            "ven.lock validation failed:\n{}",
            errors.join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_trivial_lock() {
        // Package keys are "name@version" (multi-version lockfile); see
        // from_merged_simulations. Roots use the same composite key.
        let lock = VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["left-pad@1.3.0".into()],
            packages: HashMap::from([(
                "left-pad@1.3.0".into(),
                VenLockPackage {
                    version: "1.3.0".into(),
                    integrity: None,
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
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["a".into()],
            packages: HashMap::from([(
                "a".into(),
                VenLockPackage {
                    version: "1.0.0".into(),
                    integrity: None,
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

    /// Regression test: prior to the canonical-JSON fix, two `VenLockFile`s
    /// with the same content but built in different insertion orders would
    /// hash differently because `HashMap` iteration order depends on the
    /// per-instance random hasher seed. This caused `ven sync --check` to
    /// reject every freshly-written `ven.lock`.
    #[test]
    fn hash_is_stable_across_hashmap_seeds() {
        let mk = |entries: Vec<(&str, &str)>| -> VenLockFile {
            let mut packages = HashMap::new();
            for (k, v) in entries {
                packages.insert(
                    k.to_string(),
                    VenLockPackage {
                        version: v.to_string(),
                        integrity: None,
                        metadata: None,
                    },
                );
            }
            VenLockFile {
                lock_format_version: LOCK_FORMAT_VERSION,
                ecosystem: "npm".into(),
                runtime_kind: RuntimeKind::NpmFamily,
                runtime_version: "20".into(),
                roots: vec!["a".into()],
                packages,
                edges: vec![],
                content_hash: None,
            }
        };
        let forward = mk(vec![
            ("a", "1.0.0"),
            ("b", "2.0.0"),
            ("c", "3.0.0"),
            ("d", "4.0.0"),
            ("e", "5.0.0"),
        ]);
        let reverse = mk(vec![
            ("e", "5.0.0"),
            ("d", "4.0.0"),
            ("c", "3.0.0"),
            ("b", "2.0.0"),
            ("a", "1.0.0"),
        ]);
        let h1 = compute_lock_content_hash(&forward).unwrap();
        let h2 = compute_lock_content_hash(&reverse).unwrap();
        assert_eq!(
            h1, h2,
            "hash must be independent of HashMap insertion order"
        );
    }

    /// End-to-end: write -> read -> rehash must reproduce the stamped hash.
    /// Same scenario as `ven lock` followed by `ven sync --check`.
    #[test]
    fn hash_round_trip_through_json_matches_stamp() {
        let mut packages = HashMap::new();
        for (k, v) in [("alpha", "1.2.3"), ("beta", "0.0.1"), ("gamma", "2.4.6")] {
            packages.insert(
                k.into(),
                VenLockPackage {
                    version: v.into(),
                    integrity: None,
                    metadata: None,
                },
            );
        }
        let mut lock = VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["alpha".into()],
            packages,
            edges: vec![],
            content_hash: None,
        };
        lock.content_hash = Some(compute_lock_content_hash(&lock).unwrap());
        let stamped = lock.content_hash.clone().unwrap();

        // Simulate disk round-trip.
        let on_disk = serde_json::to_string_pretty(&lock).unwrap();
        let mut read_back: VenLockFile = serde_json::from_str(&on_disk).unwrap();
        let recomputed = {
            read_back.content_hash = None;
            compute_lock_content_hash(&read_back).unwrap()
        };
        assert_eq!(stamped, recomputed);
    }

    #[test]
    fn integrity_format_accepts_npm_style() {
        assert!(integrity_format_valid("sha512-abc123=="));
        assert!(integrity_format_valid(
            "sha512-d8X4xQ+ai/q7+yyXZpQ7gZQqVZ2RZ0mKXMFb6XwH7m9c2VKL+H5GGr6Q=="
        ));
        assert!(integrity_format_valid("sha256-abc"));
        assert!(integrity_format_valid("sha384-XYZ"));
    }

    #[test]
    fn integrity_format_rejects_garbage() {
        assert!(!integrity_format_valid(""));
        assert!(!integrity_format_valid("md5-abc"));
        assert!(!integrity_format_valid("sha512-"));
        assert!(!integrity_format_valid("sha512abc"));
        assert!(!integrity_format_valid("sha512-abc!!"));
    }

    #[test]
    fn validate_rejects_bad_integrity() {
        let lock = VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["x@1.0.0".into()],
            packages: HashMap::from([(
                "x@1.0.0".into(),
                VenLockPackage {
                    version: "1.0.0".into(),
                    integrity: Some("md5-bad".into()),
                    metadata: None,
                },
            )]),
            edges: vec![],
            content_hash: None,
        };
        assert!(validate_lock_graph(&lock).is_err());
    }

    #[test]
    fn v1_lock_loads_with_no_integrity() {
        // Synthesized v1 doc on disk: no integrity field, lock_format_version = 1.
        let json = r#"{
            "lock_format_version": 1,
            "ecosystem": "npm",
            "runtime_kind": "NpmFamily",
            "runtime_version": "20",
            "roots": ["x"],
            "packages": {"x": {"version": "1.0.0"}},
            "edges": []
        }"#;
        let lock: VenLockFile = serde_json::from_str(json).unwrap();
        assert_eq!(lock.lock_format_version, 1);
        assert!(lock.packages["x"].integrity.is_none());
        assert!(lock_needs_upgrade(&lock));
        validate_lock_graph(&lock).unwrap();
    }

    /// v2 lockfile *without* a content_hash must be rejected by `read_path`.
    /// The old behavior was to silently accept it — that defeats the entire
    /// purpose of the integrity stamp (an attacker who can edit the file
    /// can also strip the hash).
    #[test]
    fn read_path_rejects_v2_lock_without_content_hash() {
        let dir = tempdir_in_target();
        let path = dir.join("ven.lock");
        let json = r#"{
            "lock_format_version": 2,
            "ecosystem": "npm",
            "runtime_kind": "NpmFamily",
            "runtime_version": "20",
            "roots": ["x"],
            "packages": {"x": {"version": "1.0.0"}},
            "edges": []
        }"#;
        std::fs::write(&path, json).unwrap();
        let err = VenLockFile::read_path(&path).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("content_hash") || msg.contains("ven lock"),
            "expected content_hash error, got: {}",
            msg
        );
    }

    /// A v2 lockfile with a tampered content_hash must still be rejected.
    #[test]
    fn read_path_rejects_v2_lock_with_tampered_hash() {
        let dir = tempdir_in_target();
        let path = dir.join("ven.lock");
        // Build a real v2 lock, stamp a *wrong* hash, write to disk.
        let lock = VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["x".into()],
            packages: HashMap::from([(
                "x".into(),
                VenLockPackage {
                    version: "1.0.0".into(),
                    integrity: None,
                    metadata: None,
                },
            )]),
            edges: vec![],
            content_hash: Some(
                "0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
        };
        std::fs::write(&path, serde_json::to_string_pretty(&lock).unwrap()).unwrap();
        let err = VenLockFile::read_path(&path).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("mismatch") || msg.contains("content_hash"),
            "expected mismatch error, got: {}",
            msg
        );
    }

    /// A correctly-stamped v2 lockfile must round-trip through read_path.
    #[test]
    fn read_path_accepts_v2_lock_with_valid_hash() {
        let dir = tempdir_in_target();
        let path = dir.join("ven.lock");
        let mut lock = VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["x".into()],
            packages: HashMap::from([(
                "x".into(),
                VenLockPackage {
                    version: "1.0.0".into(),
                    integrity: None,
                    metadata: None,
                },
            )]),
            edges: vec![],
            content_hash: None,
        };
        lock.content_hash = Some(compute_lock_content_hash(&lock).unwrap());
        std::fs::write(&path, serde_json::to_string_pretty(&lock).unwrap()).unwrap();
        let read_back = VenLockFile::read_path(&path).unwrap();
        assert_eq!(read_back.packages["x"].version, "1.0.0");
    }

    fn tempdir_in_target() -> std::path::PathBuf {
        // Use the process temp dir so each test gets a clean slate.
        let mut p = std::env::temp_dir();
        p.push(format!("ven-lock-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
