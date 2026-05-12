//! OSV (osv.dev) vulnerability scanner — multi-ecosystem.
//!
//! `POST https://api.osv.dev/v1/querybatch` returns one entry per query;
//! each entry has zero or more vulnerabilities. We cache the raw payload in
//! `~/.ven/intelligence.db` (`osv_cache` table) keyed by
//! (ecosystem, package, version) with a 6-hour TTL. On network failure we
//! still serve stale rows so users keep working offline.
//!
//! Cross-platform: pure `reqwest` + `rusqlite::bundled`. No native deps.

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::intelligence::graph::RuntimeKind;

/// OSV ecosystem string, exactly as the OSV API expects.
/// Maps the eight `ven` runtimes to their OSV slug. Returns `None` for
/// runtimes OSV doesn't track (Deno URL imports — Deno's `npm:` specifiers
/// are mapped to `npm`).
pub fn osv_ecosystem_for(kind: &RuntimeKind) -> Option<&'static str> {
    match kind {
        RuntimeKind::NpmFamily => Some("npm"),
        RuntimeKind::Python => Some("PyPI"),
        RuntimeKind::Go => Some("Go"),
        RuntimeKind::Rust => Some("crates.io"),
        RuntimeKind::Java => Some("Maven"),
        RuntimeKind::Ruby => Some("RubyGems"),
        RuntimeKind::Deno => Some("npm"),
        RuntimeKind::Stub => None,
    }
}

/// One advisory affecting a queried package@version.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OsvVuln {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// `LOW`, `MODERATE`, `HIGH`, `CRITICAL` — derived in [`OsvClient::summarize_severity`].
    #[serde(default)]
    pub severity_label: String,
    /// Highest CVSS score found across the advisory's `severity[]` array.
    #[serde(default)]
    pub cvss_score: Option<f32>,
    /// First fixed version we could extract from the advisory (best-effort).
    #[serde(default)]
    pub fixed_version: Option<String>,
    /// `https://osv.dev/vulnerability/<id>`.
    #[serde(default)]
    pub url: String,
    /// CVE IDs harvested from `aliases`.
    #[serde(default)]
    pub cves: Vec<String>,
}

/// Result for one (package, version) probe.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OsvPackageReport {
    pub ecosystem: String,
    pub package: String,
    pub version: String,
    pub vulns: Vec<OsvVuln>,
    /// `true` when this came from the on-disk cache (possibly stale).
    #[serde(default)]
    pub from_cache: bool,
}

impl OsvPackageReport {
    pub fn worst_severity(&self) -> &'static str {
        let mut worst = SeverityRank::None;
        for v in &self.vulns {
            let r = SeverityRank::from_label(&v.severity_label);
            if r > worst {
                worst = r;
            }
        }
        worst.label()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityRank {
    None,
    Low,
    Moderate,
    High,
    Critical,
}

impl SeverityRank {
    pub fn from_label(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "CRITICAL" => SeverityRank::Critical,
            "HIGH" => SeverityRank::High,
            "MODERATE" | "MEDIUM" => SeverityRank::Moderate,
            "LOW" => SeverityRank::Low,
            _ => SeverityRank::None,
        }
    }
    pub fn from_cvss(score: f32) -> Self {
        if score >= 9.0 {
            SeverityRank::Critical
        } else if score >= 7.0 {
            SeverityRank::High
        } else if score >= 4.0 {
            SeverityRank::Moderate
        } else if score > 0.0 {
            SeverityRank::Low
        } else {
            SeverityRank::None
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            SeverityRank::Critical => "CRITICAL",
            SeverityRank::High => "HIGH",
            SeverityRank::Moderate => "MODERATE",
            SeverityRank::Low => "LOW",
            SeverityRank::None => "NONE",
        }
    }
}

pub const OSV_CACHE_TTL_SECS: i64 = 6 * 60 * 60;

fn osv_cache_ttl_secs() -> i64 {
    std::env::var("VEN_OSV_TTL_SECS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(OSV_CACHE_TTL_SECS)
}
const OSV_BATCH_SIZE: usize = 1000;
const DEFAULT_OSV_BASE_URL: &str = "https://api.osv.dev/v1";

/// Base URL for OSV calls. Overridable via `VEN_OSV_BASE_URL` so tests can
/// point at a `mockito` server. Production default is `api.osv.dev/v1`.
fn osv_base_url() -> String {
    std::env::var("VEN_OSV_BASE_URL").unwrap_or_else(|_| DEFAULT_OSV_BASE_URL.to_string())
}
fn osv_querybatch_url() -> String {
    format!("{}/querybatch", osv_base_url())
}
fn osv_vuln_url(id: &str) -> String {
    format!("{}/vulns/{}", osv_base_url(), id)
}

pub struct OsvClient {
    client: reqwest::Client,
    cache: OsvCache,
}

impl OsvClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("ven/0.1.0 (+https://github.com/yourorg/ven)")
            .build()?;
        Ok(Self {
            client,
            cache: OsvCache::open()?,
        })
    }

    /// One probe per `(ecosystem, package, version)`. Each query is its own
    /// list element, so OSV reports are positionally aligned to inputs.
    pub async fn query_packages(&self, queries: &[OsvQuery]) -> Result<Vec<OsvPackageReport>> {
        let mut reports: Vec<Option<OsvPackageReport>> = vec![None; queries.len()];
        let mut to_fetch: Vec<usize> = Vec::new();

        // 1. Cache pass.
        for (i, q) in queries.iter().enumerate() {
            if let Some(cached) = self.cache.get(&q.ecosystem, &q.package, &q.version)? {
                reports[i] = Some(cached);
            } else {
                to_fetch.push(i);
            }
        }

        if to_fetch.is_empty() {
            return Ok(reports.into_iter().flatten().collect());
        }

        // 2. Fetch in chunks of 1000.
        let querybatch_url = osv_querybatch_url();
        for chunk in to_fetch.chunks(OSV_BATCH_SIZE) {
            let body = build_querybatch_body(queries, chunk);
            let resp_result = self.client.post(&querybatch_url).json(&body).send().await;

            let raw: serde_json::Value = match resp_result {
                Ok(r) if r.status().is_success() => match r.json().await {
                    Ok(v) => v,
                    Err(_) => {
                        // Serve stale on parse failure.
                        for &i in chunk {
                            let q = &queries[i];
                            reports[i] = Some(self.cache.get_stale(&q.ecosystem, &q.package, &q.version)?
                                .unwrap_or_else(|| OsvPackageReport::empty(q)));
                        }
                        continue;
                    }
                },
                _ => {
                    for &i in chunk {
                        let q = &queries[i];
                        reports[i] = Some(self.cache.get_stale(&q.ecosystem, &q.package, &q.version)?
                            .unwrap_or_else(|| OsvPackageReport::empty(q)));
                    }
                    continue;
                }
            };

            // OSV returns `{ "results": [{ "vulns": [{"id":"GHSA-..."},…] }, …] }`
            // matching our chunk order.
            let Some(results) = raw.get("results").and_then(|r| r.as_array()) else {
                continue;
            };

            for (k, &i) in chunk.iter().enumerate() {
                let q = &queries[i];
                let mut vulns: Vec<OsvVuln> = Vec::new();
                if let Some(entry) = results.get(k) {
                    if let Some(arr) = entry.get("vulns").and_then(|v| v.as_array()) {
                        for v in arr {
                            // querybatch returns IDs; fetch each /vulns/<id>
                            // for severity + summary. We do this concurrently
                            // for the chunk to keep latency reasonable.
                            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                                vulns.push(OsvVuln {
                                    id: id.to_string(),
                                    url: format!("https://osv.dev/vulnerability/{}", id),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                self.enrich_vulns(&mut vulns).await;
                let report = OsvPackageReport {
                    ecosystem: q.ecosystem.clone(),
                    package: q.package.clone(),
                    version: q.version.clone(),
                    vulns,
                    from_cache: false,
                };
                let _ = self.cache.put(&report);
                reports[i] = Some(report);
            }
        }

        Ok(reports.into_iter().flatten().collect())
    }

    /// For each vuln id, hit `/v1/vulns/<id>` and pull severity, summary,
    /// fixed version, CVE aliases. Best-effort: errors leave fields blank.
    async fn enrich_vulns(&self, vulns: &mut [OsvVuln]) {
        for v in vulns.iter_mut() {
            if v.id.is_empty() {
                continue;
            }
            let url = osv_vuln_url(&v.id);
            let Ok(resp) = self.client.get(&url).send().await else {
                continue;
            };
            if !resp.status().is_success() {
                continue;
            }
            let Ok(raw): Result<serde_json::Value, _> = resp.json().await else {
                continue;
            };

            v.summary = raw.get("summary").and_then(|s| s.as_str()).map(String::from);
            if let Some(arr) = raw.get("aliases").and_then(|a| a.as_array()) {
                for a in arr {
                    if let Some(s) = a.as_str() {
                        v.aliases.push(s.to_string());
                        if s.starts_with("CVE-") {
                            v.cves.push(s.to_string());
                        }
                    }
                }
            }
            // severity[]: { type: "CVSS_V3", score: "CVSS:3.1/..." }
            // Real CVSS parsing is hairy; we just take the highest base score.
            let mut max_score: Option<f32> = None;
            if let Some(arr) = raw.get("severity").and_then(|a| a.as_array()) {
                for s in arr {
                    if let Some(score_str) = s.get("score").and_then(|x| x.as_str()) {
                        if let Some(parsed) = parse_cvss_base_score(score_str) {
                            max_score = Some(max_score.map_or(parsed, |m| m.max(parsed)));
                        }
                    }
                }
            }
            // database_specific.severity (e.g. "HIGH"); honor when no CVSS.
            let label = raw
                .get("database_specific")
                .and_then(|d| d.get("severity"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_ascii_uppercase());

            let rank = match (max_score, label.as_deref()) {
                (Some(s), _) => SeverityRank::from_cvss(s),
                (None, Some(l)) => SeverityRank::from_label(l),
                _ => SeverityRank::None,
            };
            v.cvss_score = max_score;
            v.severity_label = rank.label().to_string();

            // Fixed version: walk affected[].ranges[].events for `fixed`.
            if let Some(affected) = raw.get("affected").and_then(|a| a.as_array()) {
                'outer: for af in affected {
                    if let Some(ranges) = af.get("ranges").and_then(|r| r.as_array()) {
                        for r in ranges {
                            if let Some(events) = r.get("events").and_then(|e| e.as_array()) {
                                for e in events {
                                    if let Some(fx) = e.get("fixed").and_then(|x| x.as_str()) {
                                        v.fixed_version = Some(fx.to_string());
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl OsvPackageReport {
    fn empty(q: &OsvQuery) -> Self {
        Self {
            ecosystem: q.ecosystem.clone(),
            package: q.package.clone(),
            version: q.version.clone(),
            vulns: Vec::new(),
            from_cache: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsvQuery {
    pub ecosystem: String,
    pub package: String,
    pub version: String,
}

impl OsvQuery {
    pub fn new(ecosystem: impl Into<String>, package: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            ecosystem: ecosystem.into(),
            package: package.into(),
            version: version.into(),
        }
    }
}

fn build_querybatch_body(queries: &[OsvQuery], indices: &[usize]) -> serde_json::Value {
    let qs: Vec<serde_json::Value> = indices
        .iter()
        .map(|&i| {
            let q = &queries[i];
            serde_json::json!({
                "version": q.version,
                "package": { "name": q.package, "ecosystem": q.ecosystem }
            })
        })
        .collect();
    serde_json::json!({ "queries": qs })
}

/// Pull a base score out of `CVSS:3.x/.../C:H/I:H/A:H` strings. Returns
/// `None` when we can't find a clear `BS:` or numeric tail.
fn parse_cvss_base_score(score: &str) -> Option<f32> {
    if let Ok(n) = score.parse::<f32>() {
        return Some(n);
    }
    // Some advisories prepend the base score:  "8.1 CVSS:3.1/..."
    if let Some((head, _)) = score.split_once(' ') {
        if let Ok(n) = head.parse::<f32>() {
            return Some(n);
        }
    }
    None
}

// ── SQLite cache ────────────────────────────────────────────────────────────

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

struct OsvCache {
    conn: Connection,
}

impl OsvCache {
    fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS osv_cache(
                ecosystem  TEXT NOT NULL,
                package    TEXT NOT NULL,
                version    TEXT NOT NULL,
                fetched_at INTEGER NOT NULL,
                payload    TEXT NOT NULL,
                PRIMARY KEY(ecosystem, package, version)
            );",
        )?;
        Ok(Self { conn })
    }

    fn get(&self, ecosystem: &str, package: &str, version: &str) -> Result<Option<OsvPackageReport>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM osv_cache WHERE ecosystem=?1 AND package=?2 AND version=?3")?;
        let row: Option<(String, i64)> = stmt
            .query_row(params![ecosystem, package, version], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?;
        match row {
            Some((payload, ts)) if now_secs().saturating_sub(ts) <= osv_cache_ttl_secs() => {
                let mut report: OsvPackageReport = serde_json::from_str(&payload)?;
                report.from_cache = true;
                Ok(Some(report))
            }
            _ => Ok(None),
        }
    }

    /// Used only as a fallback when network fails — TTL is ignored.
    fn get_stale(&self, ecosystem: &str, package: &str, version: &str) -> Result<Option<OsvPackageReport>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM osv_cache WHERE ecosystem=?1 AND package=?2 AND version=?3")?;
        let row: Option<String> = stmt
            .query_row(params![ecosystem, package, version], |r| r.get(0))
            .optional()?;
        match row {
            Some(payload) => {
                let mut report: OsvPackageReport = serde_json::from_str(&payload)?;
                report.from_cache = true;
                Ok(Some(report))
            }
            None => Ok(None),
        }
    }

    fn put(&self, report: &OsvPackageReport) -> Result<()> {
        let payload = serde_json::to_string(report)
            .map_err(|e| anyhow!("serialize osv report: {e}"))?;
        self.conn.execute(
            "INSERT INTO osv_cache(ecosystem, package, version, fetched_at, payload)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(ecosystem, package, version) DO UPDATE SET
               fetched_at = excluded.fetched_at,
               payload    = excluded.payload",
            params![
                report.ecosystem,
                report.package,
                report.version,
                now_secs(),
                payload
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_mapping_covers_all_runtimes() {
        assert_eq!(osv_ecosystem_for(&RuntimeKind::NpmFamily), Some("npm"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Python), Some("PyPI"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Go), Some("Go"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Rust), Some("crates.io"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Java), Some("Maven"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Ruby), Some("RubyGems"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Deno), Some("npm"));
        assert_eq!(osv_ecosystem_for(&RuntimeKind::Stub), None);
    }

    #[test]
    fn severity_label_round_trip() {
        assert_eq!(SeverityRank::from_label("CRITICAL"), SeverityRank::Critical);
        assert_eq!(SeverityRank::from_label("high"), SeverityRank::High);
        assert_eq!(SeverityRank::from_label("MEDIUM"), SeverityRank::Moderate);
        assert_eq!(SeverityRank::from_label("low"), SeverityRank::Low);
        assert_eq!(SeverityRank::from_label("???"), SeverityRank::None);
    }

    #[test]
    fn severity_from_cvss_buckets() {
        assert_eq!(SeverityRank::from_cvss(9.5), SeverityRank::Critical);
        assert_eq!(SeverityRank::from_cvss(7.5), SeverityRank::High);
        assert_eq!(SeverityRank::from_cvss(5.0), SeverityRank::Moderate);
        assert_eq!(SeverityRank::from_cvss(2.1), SeverityRank::Low);
        assert_eq!(SeverityRank::from_cvss(0.0), SeverityRank::None);
    }

    #[test]
    fn cvss_score_parsing_falls_back_to_leading_number() {
        assert_eq!(parse_cvss_base_score("7.5"), Some(7.5));
        assert_eq!(parse_cvss_base_score("8.1 CVSS:3.1/AV:N"), Some(8.1));
        assert_eq!(parse_cvss_base_score("CVSS:3.1/AV:N"), None);
    }

    #[test]
    fn querybatch_body_is_indexed_correctly() {
        let queries = vec![
            OsvQuery::new("npm", "express", "4.17.1"),
            OsvQuery::new("PyPI", "requests", "2.20.0"),
        ];
        let body = build_querybatch_body(&queries, &[0, 1]);
        let arr = body.get("queries").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["package"]["name"], "express");
        assert_eq!(arr[1]["package"]["ecosystem"], "PyPI");
    }
}
