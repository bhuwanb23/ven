//! Persist intelligence snapshots and lock/cache tables (SQLite).

use anyhow::Result;
use rusqlite::Connection;
use serde_json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::ven_home::ven_home;
use crate::intelligence::graph::SimulationResult;
use crate::intelligence::ven_lock::VenLockFile;

fn db_path() -> PathBuf {
    ven_home().join("intelligence.db")
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct IntelligenceStore {
    conn: Connection,
}

impl IntelligenceStore {
    pub fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                project_key TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )?;
        // Best-effort migration for graph hash column
        let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN graph_hash TEXT", []);
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS package_cache (
                package_name TEXT NOT NULL,
                package_version TEXT NOT NULL,
                ecosystem TEXT NOT NULL,
                metadata_json TEXT NOT NULL,
                cached_at INTEGER NOT NULL,
                PRIMARY KEY (package_name, package_version, ecosystem)
            );
            CREATE TABLE IF NOT EXISTS dependency_cache (
                from_package TEXT NOT NULL,
                from_version TEXT NOT NULL,
                to_package TEXT NOT NULL,
                to_constraint TEXT NOT NULL,
                constraint_type TEXT NOT NULL,
                ecosystem TEXT NOT NULL,
                cached_at INTEGER NOT NULL,
                PRIMARY KEY (from_package, from_version, to_package, to_constraint, constraint_type, ecosystem)
            );
            CREATE TABLE IF NOT EXISTS lock_validations (
                project_key TEXT PRIMARY KEY,
                validated_at INTEGER NOT NULL,
                graph_hash TEXT NOT NULL,
                lock_content_hash TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn save(&self, project_key: &str, result: &SimulationResult) -> Result<()> {
        let payload = serde_json::to_string(result)?;
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO snapshots (project_key, payload, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(project_key) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at",
            rusqlite::params![project_key, payload, now],
        )?;
        Ok(())
    }

    pub fn save_with_graph_hash(
        &self,
        project_key: &str,
        result: &SimulationResult,
        graph_hash: &str,
    ) -> Result<()> {
        let payload = serde_json::to_string(result)?;
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO snapshots (project_key, payload, updated_at, graph_hash) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_key) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at, graph_hash = excluded.graph_hash",
            rusqlite::params![project_key, payload, now, graph_hash],
        )?;
        Ok(())
    }

    pub fn load(&self, project_key: &str) -> Result<Option<SimulationResult>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM snapshots WHERE project_key = ?1")?;
        let mut rows = stmt.query(rusqlite::params![project_key])?;
        if let Some(row) = rows.next()? {
            let s: String = row.get(0)?;
            Ok(Some(serde_json::from_str(&s)?))
        } else {
            Ok(None)
        }
    }

    /// Cache lockfile packages (TTL enforced on read by callers; 1 hour convention).
    pub fn upsert_packages_from_lock(&self, lock: &VenLockFile) -> Result<()> {
        let now = now_secs();
        let eco = lock.ecosystem.as_str();
        for (name, pkg) in &lock.packages {
            let meta = serde_json::to_string(&pkg.metadata)?;
            self.conn.execute(
                "INSERT INTO package_cache (package_name, package_version, ecosystem, metadata_json, cached_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(package_name, package_version, ecosystem) DO UPDATE SET
                   metadata_json = excluded.metadata_json,
                   cached_at = excluded.cached_at",
                rusqlite::params![name, pkg.version, eco, meta, now],
            )?;
        }
        Ok(())
    }

    pub fn upsert_dependencies_from_lock(&self, lock: &VenLockFile) -> Result<()> {
        let now = now_secs();
        let eco = lock.ecosystem.as_str();
        for e in &lock.edges {
            let Some((from_pkg, from_ver)) = e.from.rsplit_once('@') else {
                continue;
            };
            let Some((to_pkg, _)) = e.to.rsplit_once('@') else {
                continue;
            };
            let kind = format!("{:?}", e.kind);
            self.conn.execute(
                "INSERT INTO dependency_cache (from_package, from_version, to_package, to_constraint, constraint_type, ecosystem, cached_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(from_package, from_version, to_package, to_constraint, constraint_type, ecosystem) DO UPDATE SET
                   cached_at = excluded.cached_at",
                rusqlite::params![from_pkg, from_ver, to_pkg, e.constraint, kind, eco, now],
            )?;
        }
        Ok(())
    }

    pub fn record_lock_validation(
        &self,
        project_key: &str,
        graph_hash: &str,
        lock_content_hash: &str,
    ) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO lock_validations (project_key, validated_at, graph_hash, lock_content_hash)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_key) DO UPDATE SET
               validated_at = excluded.validated_at,
               graph_hash = excluded.graph_hash,
               lock_content_hash = excluded.lock_content_hash",
            rusqlite::params![project_key, now, graph_hash, lock_content_hash],
        )?;
        Ok(())
    }
}

/// True if `cached_at` is within the last `ttl_secs` seconds.
pub fn cache_entry_fresh(cached_at: i64, ttl_secs: u64) -> bool {
    let now = now_secs();
    now.saturating_sub(cached_at) <= ttl_secs as i64
}

pub const PACKAGE_CACHE_TTL_SECS: u64 = 3600;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::graph::{IntelGraph, RuntimeKind, SimulationResult};
    use crate::intelligence::ven_lock::{VenLockEdge, VenLockFile, VenLockPackage, LOCK_FORMAT_VERSION};
    use tempfile::TempDir;

    fn temp_ven_home() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("VEN_HOME", dir.path());
        dir
    }

    fn mk_simulation_result() -> SimulationResult {
        SimulationResult {
            compatible: true,
            graph: IntelGraph {
                runtime_kind: RuntimeKind::NpmFamily,
                runtime_version: "20".into(),
                nodes: BTreeMap::new(),
                edges: vec![],
            },
            conflict_chains: vec![],
            engine_incompatibilities: vec![],
            suggestions: vec![],
            warnings: vec![],
        }
    }

    fn mk_lock() -> VenLockFile {
        let mut packages = HashMap::new();
        packages.insert(
            "express".into(),
            VenLockPackage {
                version: "4.18.0".into(),
                integrity: None,
                metadata: Some(serde_json::json!({"description": "Express web framework"})),
            },
        );
        VenLockFile {
            lock_format_version: LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["express@4.18.0".into()],
            packages,
            edges: vec![VenLockEdge {
                from: "express@4.18.0".into(),
                to: "body-parser@1.20.0".into(),
                constraint: "^1.19.0".into(),
                kind: Default::default(),
            }],
            content_hash: None,
        }
    }

    #[test]
    fn open_creates_db_and_tables() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        let tables: Vec<String> = store
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"snapshots".into()));
        assert!(tables.contains(&"package_cache".into()));
        assert!(tables.contains(&"dependency_cache".into()));
        assert!(tables.contains(&"lock_validations".into()));
    }

    #[test]
    fn save_and_load_round_trip() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        let result = mk_simulation_result();
        store.save("test-project", &result).unwrap();

        let loaded = store.load("test-project").unwrap().unwrap();
        assert_eq!(loaded.compatible, true);
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn load_returns_none_for_missing_key() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        let loaded = store.load("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn save_overwrites_existing_snapshot() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        let mut result1 = mk_simulation_result();
        result1.warnings = vec!["first".into()];
        store.save("key", &result1).unwrap();

        let mut result2 = mk_simulation_result();
        result2.warnings = vec!["second".into()];
        store.save("key", &result2).unwrap();

        let loaded = store.load("key").unwrap().unwrap();
        assert_eq!(loaded.warnings, vec!["second".into()]);
    }

    #[test]
    fn save_with_graph_hash_round_trip() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        let result = mk_simulation_result();
        store
            .save_with_graph_hash("proj", &result, "abc123")
            .unwrap();

        let loaded = store.load("proj").unwrap().unwrap();
        assert_eq!(loaded.compatible, true);

        let mut stmt = store
            .conn
            .prepare("SELECT graph_hash FROM snapshots WHERE project_key = ?1")
            .unwrap();
        let hash: String = stmt
            .query_row(rusqlite::params!["proj"], |row| row.get(0))
            .unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn package_cache_upsert_and_retrieve() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();
        let lock = mk_lock();

        store.upsert_packages_from_lock(&lock).unwrap();

        let mut stmt = store
            .conn
            .prepare("SELECT package_name, package_version, ecosystem, metadata_json FROM package_cache WHERE package_name = ?1")
            .unwrap();
        let (name, version, eco, meta): (String, String, String, String) = stmt
            .query_row(rusqlite::params!["express"], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap();

        assert_eq!(name, "express");
        assert_eq!(version, "4.18.0");
        assert_eq!(eco, "npm");
        let parsed: serde_json::Value = serde_json::from_str(&meta).unwrap();
        assert_eq!(
            parsed["description"].as_str().unwrap(),
            "Express web framework"
        );
    }

    #[test]
    fn package_cache_upsert_updates_existing() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();
        let mut lock = mk_lock();
        store.upsert_packages_from_lock(&lock).unwrap();

        lock.packages.insert(
            "express".into(),
            VenLockPackage {
                version: "4.18.0".into(),
                integrity: None,
                metadata: Some(serde_json::json!({"description": "Updated"})),
            },
        );
        store.upsert_packages_from_lock(&lock).unwrap();

        let mut stmt = store
            .conn
            .prepare("SELECT metadata_json FROM package_cache WHERE package_name = ?1")
            .unwrap();
        let meta: String = stmt
            .query_row(rusqlite::params!["express"], |row| row.get(0))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&meta).unwrap();
        assert_eq!(parsed["description"].as_str().unwrap(), "Updated");
    }

    #[test]
    fn dependency_cache_upsert_and_retrieve() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();
        let lock = mk_lock();

        store.upsert_dependencies_from_lock(&lock).unwrap();

        let mut stmt = store
            .conn
            .prepare("SELECT from_package, from_version, to_package, constraint_type FROM dependency_cache")
            .unwrap();
        let (from_pkg, from_ver, to_pkg, kind): (String, String, String, String) = stmt
            .query_row([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .unwrap();

        assert_eq!(from_pkg, "express");
        assert_eq!(from_ver, "4.18.0");
        assert_eq!(to_pkg, "body-parser");
        assert_eq!(kind, "Dependency");
    }

    #[test]
    fn dependency_cache_ignores_malformed_edges() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();
        let mut lock = mk_lock();
        lock.edges.push(VenLockEdge {
            from: "no-at-sign".into(),
            to: "also-bad".into(),
            constraint: "^1.0.0".into(),
            kind: Default::default(),
        });

        store.upsert_dependencies_from_lock(&lock).unwrap();

        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM dependency_cache", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn cleanup_removes_expired_cache_entries() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        let old_time = now_secs() - (PACKAGE_CACHE_TTL_SECS as i64 + 1);
        store.conn.execute(
            "INSERT INTO package_cache (package_name, package_version, ecosystem, metadata_json, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["old-pkg", "1.0.0", "npm", "{}", old_time],
        ).unwrap();
        store.conn.execute(
            "INSERT INTO package_cache (package_name, package_version, ecosystem, metadata_json, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["new-pkg", "2.0.0", "npm", "{}", now_secs()],
        ).unwrap();

        store.conn.execute(
            "DELETE FROM package_cache WHERE cached_at < ?1",
            rusqlite::params![now_secs() - PACKAGE_CACHE_TTL_SECS as i64],
        ).unwrap();

        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM package_cache", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);

        let name: String = store
            .conn
            .query_row(
                "SELECT package_name FROM package_cache",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "new-pkg");
    }

    #[test]
    fn cache_entry_fresh_respects_ttl() {
        assert!(cache_entry_fresh(now_secs(), PACKAGE_CACHE_TTL_SECS));
        assert!(cache_entry_fresh(
            now_secs() - 100,
            PACKAGE_CACHE_TTL_SECS
        ));
        assert!(!cache_entry_fresh(
            now_secs() - PACKAGE_CACHE_TTL_SECS as i64 - 1,
            PACKAGE_CACHE_TTL_SECS
        ));
    }

    #[test]
    fn lock_validation_round_trip() {
        let _dir = temp_ven_home();
        let store = IntelligenceStore::open().unwrap();

        store
            .record_lock_validation("proj", "graph-hash-1", "lock-hash-1")
            .unwrap();

        let mut stmt = store
            .conn
            .prepare("SELECT graph_hash, lock_content_hash FROM lock_validations WHERE project_key = ?1")
            .unwrap();
        let (gh, lch): (String, String) = stmt
            .query_row(rusqlite::params!["proj"], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();
        assert_eq!(gh, "graph-hash-1");
        assert_eq!(lch, "lock-hash-1");
    }
}
