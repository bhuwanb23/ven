use anyhow::{Result, anyhow};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// npm Registry client with SQLite caching
pub struct NpmRegistry {
    cache: RegistryCache,
    client: reqwest::Client,
}

/// Package metadata from npm registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    #[serde(rename = "dist-tags", default)]
    pub dist_tags: HashMap<String, String>,
    #[serde(default)]
    pub versions: HashMap<String, VersionMetadata>,
    #[serde(default)]
    pub time: HashMap<String, String>,
}

/// Version-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub version: String,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies", default)]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies", default)]
    pub peer_dependencies: Option<HashMap<String, String>>,
    #[serde(default)]
    pub engines: Option<Engines>,
    #[serde(default)]
    pub deprecated: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub dist: Option<DistInfo>,
}

/// Distribution information (size, integrity, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistInfo {
    #[serde(rename = "unpackedSize", default)]
    pub unpacked_size: Option<u64>,
    #[serde(default)]
    pub integrity: Option<String>,
}

/// Engine requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engines {
    #[serde(default)]
    pub node: Option<String>,
    #[serde(default)]
    pub npm: Option<String>,
}

/// SQLite cache for npm metadata
struct RegistryCache {
    db_path: PathBuf,
    conn: Connection,
    cache_ttl_seconds: u64,
}

impl NpmRegistry {
    pub fn new() -> Result<Self> {
        let cache = RegistryCache::new()?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("ven/0.1.0 (Node.js Version Manager)")
            .build()?;

        Ok(Self { cache, client })
    }

    /// Fetch complete package metadata (all versions)
    pub async fn fetch_package_metadata(&self, name: &str) -> Result<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get(name)? {
            return Ok(cached);
        }

        // Fetch from npm registry
        let url = format!("https://registry.npmjs.org/{}", name);
        println!("  {} Fetching {} from npm registry...", "[HTTP]".cyan(), name);

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 404 {
            return Err(anyhow!("Package '{}' not found on npm", name));
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch {}: HTTP {}",
                name,
                response.status()
            ));
        }

        let metadata: PackageMetadata = response.json().await?;

        // Cache the result
        self.cache.set(name, &metadata)?;

        Ok(metadata)
    }

    /// Fetch metadata for a specific version
    pub async fn fetch_version_metadata(&self, name: &str, version: &str) -> Result<VersionMetadata> {
        // First try to get from cached package metadata
        if let Some(package) = self.cache.get(name)? {
            if let Some(version_meta) = package.versions.get(version) {
                return Ok(version_meta.clone());
            }
        }

        // Fetch specific version from npm
        let url = format!("https://registry.npmjs.org/{}/{}", name, version);
        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 404 {
            return Err(anyhow!(
                "Version {} of package '{}' not found",
                version,
                name
            ));
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch {}@{}: HTTP {}",
                name,
                version,
                response.status()
            ));
        }

        let metadata: VersionMetadata = response.json().await?;
        Ok(metadata)
    }

    /// Check if package exists (without downloading full metadata)
    pub async fn package_exists(&self, name: &str) -> Result<bool> {
        match self.fetch_package_metadata(name).await {
            Ok(_) => Ok(true),
            Err(e) if e.to_string().contains("not found") => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get latest version of a package
    pub async fn get_latest_version(&self, name: &str) -> Result<String> {
        let metadata = self.fetch_package_metadata(name).await?;
        
        metadata.dist_tags.get("latest")
            .cloned()
            .ok_or_else(|| anyhow!("No 'latest' tag found for {}", name))
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&self) -> Result<()> {
        self.cache.cleanup_expired()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<CacheStats> {
        self.cache.stats()
    }
}

/// Cache statistics
pub struct CacheStats {
    pub total_packages: usize,
    pub expired_packages: usize,
    pub cache_size_mb: f64,
}

impl RegistryCache {
    fn new() -> Result<Self> {
        // Create cache directory
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Cannot find home directory"))?
            .join(".ven")
            .join("cache");

        std::fs::create_dir_all(&cache_dir)?;

        let db_path = cache_dir.join("registry.db");

        // Open or create database
        let conn = Connection::open(&db_path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS packages (
                name TEXT PRIMARY KEY,
                metadata TEXT NOT NULL,
                fetched_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create index for expiry cleanup
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_expires ON packages(expires_at)",
            [],
        )?;

        let cache = Self {
            db_path,
            conn,
            cache_ttl_seconds: 24 * 60 * 60, // 24 hours
        };

        // Cleanup expired entries on startup
        cache.cleanup_expired()?;

        Ok(cache)
    }

    fn get(&self, name: &str) -> Result<Option<PackageMetadata>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self.conn.prepare(
            "SELECT metadata FROM packages WHERE name = ?1 AND expires_at > ?2"
        )?;

        let metadata_json: Option<String> = stmt
            .query_row((name, now), |row| row.get(0))
            .optional()?;

        match metadata_json {
            Some(json) => {
                let metadata: PackageMetadata = serde_json::from_str(&json)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    fn set(&self, name: &str, metadata: &PackageMetadata) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expires_at = now + self.cache_ttl_seconds as i64;
        let metadata_json = serde_json::to_string(metadata)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO packages (name, metadata, fetched_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)",
            [name, &metadata_json, &now.to_string(), &expires_at.to_string()],
        )?;

        Ok(())
    }

    fn cleanup_expired(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let deleted = self.conn.execute(
            "DELETE FROM packages WHERE expires_at <= ?1",
            [&now.to_string()],
        )?;

        if deleted > 0 {
            println!("  {} Cleaned up {} expired cache entries", "🧹".cyan(), deleted);
        }

        Ok(())
    }

    fn stats(&self) -> Result<CacheStats> {
        let mut total_stmt = self.conn.prepare("SELECT COUNT(*) FROM packages")?;
        let total_packages: usize = total_stmt.query_row([], |row| row.get(0))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut expired_stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM packages WHERE expires_at <= ?1"
        )?;
        let expired_packages: usize = expired_stmt.query_row([&now.to_string()], |row| row.get(0))?;

        // Get file size
        let cache_size_mb = if self.db_path.exists() {
            let metadata = std::fs::metadata(&self.db_path)?;
            metadata.len() as f64 / (1024.0 * 1024.0)
        } else {
            0.0
        };

        Ok(CacheStats {
            total_packages,
            expired_packages,
            cache_size_mb,
        })
    }
}

// Add Colorize trait for println! macros
use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_metadata() {
        // Test JSON parsing
        let json = r#"{
            "name": "express",
            "dist-tags": {"latest": "4.18.2"},
            "versions": {}
        }"#;

        let metadata: PackageMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.name, "express");
        assert_eq!(metadata.dist_tags["latest"], "4.18.2");
    }

    #[test]
    fn test_parse_version_metadata() {
        let json = r#"{
            "version": "4.18.2",
            "dependencies": {
                "body-parser": "^1.20.0"
            },
            "engines": {
                "node": ">= 0.10.0"
            }
        }"#;

        let metadata: VersionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.version, "4.18.2");
        assert!(metadata.dependencies.is_some());
        assert!(metadata.engines.is_some());
    }
}
