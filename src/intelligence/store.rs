//! Persist intelligence snapshots and lock/cache tables (SQLite).

use anyhow::Result;
use rusqlite::Connection;
use serde_json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::intelligence::graph::SimulationResult;
use crate::intelligence::ven_lock::VenLockFile;

fn db_path() -> PathBuf {
    std::env::var("VEN_STORAGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().expect("home").join(".ven"))
        .join("intelligence.db")
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
