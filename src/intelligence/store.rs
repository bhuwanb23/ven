//! Persist last intelligence snapshot per project (SQLite).

use anyhow::Result;
use rusqlite::Connection;
use serde_json;
use std::path::PathBuf;

use crate::intelligence::graph::SimulationResult;

fn db_path() -> PathBuf {
    std::env::var("VEN_STORAGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().expect("home").join(".ven"))
        .join("intelligence.db")
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
        Ok(Self { conn })
    }

    pub fn save(&self, project_key: &str, result: &SimulationResult) -> Result<()> {
        let payload = serde_json::to_string(result)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO snapshots (project_key, payload, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(project_key) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at",
            rusqlite::params![project_key, payload, now],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
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
}
