//! endoflife.date client — runtime EOL alerts.
//!
//! Each ecosystem maps to a slug (e.g. `nodejs`, `python`). One JSON GET per
//! product fetches every cycle; we pick the one whose `cycle` matches the
//! active major version. Cached for 24 hours in `intelligence.db`
//! (`eol_cache` table). On network failure we serve stale rows so users
//! aren't blocked offline.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::intelligence::graph::RuntimeKind;

/// Map a `RuntimeKind` to its endoflife.date product slug.
pub fn endoflife_slug_for(kind: &RuntimeKind) -> Option<&'static str> {
    match kind {
        RuntimeKind::NpmFamily => Some("nodejs"),
        RuntimeKind::Python => Some("python"),
        RuntimeKind::Go => Some("go"),
        RuntimeKind::Rust => Some("rust"),
        RuntimeKind::Java => Some("java"),
        RuntimeKind::Ruby => Some("ruby"),
        RuntimeKind::Deno => Some("deno"),
        RuntimeKind::Stub => None,
    }
}

/// Same map as [`endoflife_slug_for`] but keyed off our `[runtime].<key>`
/// name strings (so the CLI layer doesn't need a `RuntimeKind`).
pub fn endoflife_slug_for_runtime_name(name: &str) -> Option<&'static str> {
    match name {
        "node" => Some("nodejs"),
        "bun" => Some("bun"),
        "python" => Some("python"),
        "go" => Some("go"),
        "rust" => Some("rust"),
        "java" => Some("java"),
        "ruby" => Some("ruby"),
        "deno" => Some("deno"),
        _ => None,
    }
}

/// One row from the endoflife.date cycle list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EolCycle {
    pub cycle: String,
    /// Date `YYYY-MM-DD` or boolean (`true` = already EOL, `false` = no date set).
    #[serde(default)]
    pub eol: serde_json::Value,
    /// Active LTS / regular support end (same value space as `eol`).
    #[serde(default)]
    pub support: serde_json::Value,
    #[serde(default)]
    pub latest: Option<String>,
    #[serde(default, rename = "releaseDate")]
    pub release_date: Option<String>,
    #[serde(default, rename = "lts")]
    pub lts: serde_json::Value,
}

/// Status of *one* configured runtime.
#[derive(Debug, Clone, Serialize)]
pub struct EolReport {
    pub product: String,
    pub configured_version: String,
    pub matched_cycle: Option<String>,
    pub latest: Option<String>,
    pub eol_date: Option<String>,
    /// `true` if `eol_date` is in the past (or `eol == true`).
    pub eol_passed: bool,
    /// Days until EOL (negative if already passed). `None` when no date.
    pub days_until_eol: Option<i64>,
    pub support_passed: bool,
    pub days_until_support_end: Option<i64>,
    pub from_cache: bool,
    pub source_url: String,
}

pub const EOL_CACHE_TTL_SECS: i64 = 24 * 60 * 60;

pub struct EndOfLifeClient {
    client: reqwest::Client,
    cache: EolCache,
}

impl EndOfLifeClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("ven/0.1.0 (+https://github.com/bhuwanb23/ven)")
            .build()?;
        Ok(Self {
            client,
            cache: EolCache::open()?,
        })
    }

    /// Fetch one product's full cycle list (cached) and find the cycle that
    /// best matches `version` (major prefix or exact).
    pub async fn report(&self, product: &str, version: &str) -> Result<EolReport> {
        let cycles = self.fetch_cycles(product).await?;
        let from_cache = cycles.from_cache;
        let mut report = EolReport {
            product: product.to_string(),
            configured_version: version.to_string(),
            matched_cycle: None,
            latest: None,
            eol_date: None,
            eol_passed: false,
            days_until_eol: None,
            support_passed: false,
            days_until_support_end: None,
            from_cache,
            source_url: format!("https://endoflife.date/{}", product),
        };

        let Some(cycle) = pick_cycle(&cycles.list, version) else {
            return Ok(report);
        };
        report.matched_cycle = Some(cycle.cycle.clone());
        report.latest = cycle.latest.clone();

        let (eol_date_opt, eol_passed) = parse_eol_field(&cycle.eol, today_ymd());
        report.eol_date = eol_date_opt.clone();
        report.eol_passed = eol_passed;
        if let Some(d) = eol_date_opt {
            report.days_until_eol = days_between(today_ymd(), &d);
        }
        let (support_date_opt, support_passed) = parse_eol_field(&cycle.support, today_ymd());
        report.support_passed = support_passed;
        if let Some(d) = support_date_opt {
            report.days_until_support_end = days_between(today_ymd(), &d);
        }

        Ok(report)
    }

    async fn fetch_cycles(&self, product: &str) -> Result<CyclesPayload> {
        if let Some(payload) = self.cache.get(product)? {
            return Ok(payload);
        }
        let base = std::env::var("VEN_EOL_BASE_URL")
            .unwrap_or_else(|_| "https://endoflife.date/api".to_string());
        let url = format!("{}/{}.json", base, product);
        let resp = self.client.get(&url).send().await;
        match resp {
            Ok(r) if r.status().is_success() => match r.json::<Vec<EolCycle>>().await {
                Ok(list) => {
                    let payload = CyclesPayload {
                        list,
                        from_cache: false,
                    };
                    let _ = self.cache.put(product, &payload.list);
                    Ok(payload)
                }
                Err(_) => self.cache.get_stale(product)?.ok_or_else(|| {
                    anyhow::anyhow!("Failed to parse endoflife response for {}", product)
                }),
            },
            _ => self.cache.get_stale(product)?.ok_or_else(|| {
                anyhow::anyhow!("endoflife.date offline and no cache for {}", product)
            }),
        }
    }
}

struct CyclesPayload {
    list: Vec<EolCycle>,
    from_cache: bool,
}

/// Pick the cycle whose `cycle` is the most-specific prefix of `version`.
fn pick_cycle<'a>(cycles: &'a [EolCycle], version: &str) -> Option<&'a EolCycle> {
    if cycles.is_empty() {
        return None;
    }
    let v = version.trim();
    // Try exact match first, then major.minor, then bare major.
    let candidates: Vec<String> = vec![
        v.to_string(),
        v.split('.').take(2).collect::<Vec<_>>().join("."),
        v.split('.').next().unwrap_or(v).to_string(),
    ];
    for c in candidates {
        if let Some(found) = cycles.iter().find(|x| x.cycle == c) {
            return Some(found);
        }
    }
    None
}

/// Returns `(date_string, passed_bool)`.
/// endoflife.date uses `YYYY-MM-DD` strings or booleans.
fn parse_eol_field(value: &serde_json::Value, today: &str) -> (Option<String>, bool) {
    if let Some(b) = value.as_bool() {
        return (None, b);
    }
    if let Some(s) = value.as_str() {
        let passed = s <= today;
        return (Some(s.to_string()), passed);
    }
    (None, false)
}

fn today_ymd() -> &'static str {
    // Avoid pulling in `chrono`; for "is date in the past" string compare on
    // ISO dates (`YYYY-MM-DD`) is sufficient and timezone-agnostic enough.
    static TODAY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TODAY
        .get_or_init(|| {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            ymd_from_unix(secs)
        })
        .as_str()
}

/// Civil-date conversion (Howard Hinnant). Pure integer math, no chrono.
fn ymd_from_unix(secs: i64) -> String {
    let days = secs.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", year, m, d)
}

fn ymd_to_days(ymd: &str) -> Option<i64> {
    let mut parts = ymd.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    let yy = if m <= 2 { y - 1 } else { y };
    let era = yy.div_euclid(400);
    let yoe = yy - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn days_between(a: &str, b: &str) -> Option<i64> {
    let a = ymd_to_days(a)?;
    let b = ymd_to_days(b)?;
    Some(b - a)
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

struct EolCache {
    conn: Connection,
}

impl EolCache {
    fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS eol_cache(
                product    TEXT PRIMARY KEY,
                fetched_at INTEGER NOT NULL,
                payload    TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    fn get(&self, product: &str) -> Result<Option<CyclesPayload>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, fetched_at FROM eol_cache WHERE product=?1")?;
        let row: Option<(String, i64)> = stmt
            .query_row(params![product], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()?;
        match row {
            Some((payload, ts)) if now_secs().saturating_sub(ts) <= EOL_CACHE_TTL_SECS => {
                let list: Vec<EolCycle> = serde_json::from_str(&payload)?;
                Ok(Some(CyclesPayload {
                    list,
                    from_cache: true,
                }))
            }
            _ => Ok(None),
        }
    }

    fn get_stale(&self, product: &str) -> Result<Option<CyclesPayload>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM eol_cache WHERE product=?1")?;
        let row: Option<String> = stmt.query_row(params![product], |r| r.get(0)).optional()?;
        match row {
            Some(payload) => {
                let list: Vec<EolCycle> = serde_json::from_str(&payload)?;
                Ok(Some(CyclesPayload {
                    list,
                    from_cache: true,
                }))
            }
            None => Ok(None),
        }
    }

    fn put(&self, product: &str, list: &[EolCycle]) -> Result<()> {
        let payload = serde_json::to_string(list)?;
        self.conn.execute(
            "INSERT INTO eol_cache(product, fetched_at, payload)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(product) DO UPDATE SET fetched_at=excluded.fetched_at, payload=excluded.payload",
            params![product, now_secs(), payload],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_map_covers_all_runtimes() {
        assert_eq!(endoflife_slug_for(&RuntimeKind::NpmFamily), Some("nodejs"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Python), Some("python"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Go), Some("go"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Rust), Some("rust"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Java), Some("java"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Ruby), Some("ruby"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Deno), Some("deno"));
        assert_eq!(endoflife_slug_for(&RuntimeKind::Stub), None);
    }

    #[test]
    fn slug_for_runtime_name_handles_bun() {
        assert_eq!(endoflife_slug_for_runtime_name("bun"), Some("bun"));
        assert_eq!(endoflife_slug_for_runtime_name("node"), Some("nodejs"));
        assert_eq!(endoflife_slug_for_runtime_name("nope"), None);
    }

    fn cycle(name: &str, eol: serde_json::Value) -> EolCycle {
        EolCycle {
            cycle: name.into(),
            eol,
            support: serde_json::Value::Bool(false),
            latest: None,
            release_date: None,
            lts: serde_json::Value::Bool(false),
        }
    }

    #[test]
    fn pick_cycle_prefers_exact_then_minor_then_major() {
        let cycles = vec![
            cycle("18", serde_json::json!("2025-04-30")),
            cycle("20", serde_json::json!("2026-04-30")),
            cycle("20.10", serde_json::json!("2025-10-01")),
        ];
        // bare major
        assert_eq!(pick_cycle(&cycles, "18.16.0").unwrap().cycle, "18");
        // major.minor preferred over major
        assert_eq!(pick_cycle(&cycles, "20.10.5").unwrap().cycle, "20.10");
        // exact major fallback
        assert_eq!(pick_cycle(&cycles, "20.11.0").unwrap().cycle, "20");
    }

    #[test]
    fn parse_eol_field_handles_strings_and_bools() {
        let (date, passed) = parse_eol_field(&serde_json::json!("2020-01-01"), "2026-05-12");
        assert_eq!(date.as_deref(), Some("2020-01-01"));
        assert!(passed);

        let (date, passed) = parse_eol_field(&serde_json::json!("2099-01-01"), "2026-05-12");
        assert_eq!(date.as_deref(), Some("2099-01-01"));
        assert!(!passed);

        let (date, passed) = parse_eol_field(&serde_json::json!(true), "2026-05-12");
        assert_eq!(date, None);
        assert!(passed);

        let (date, passed) = parse_eol_field(&serde_json::json!(false), "2026-05-12");
        assert_eq!(date, None);
        assert!(!passed);
    }

    #[test]
    fn date_round_trip_civil_calendar() {
        // 2026-05-12 ≈ unix 1778544000
        let s = ymd_from_unix(1778544000);
        assert_eq!(s, "2026-05-12");
        // and back
        let d = ymd_to_days("2026-05-12").unwrap();
        let s2 = ymd_from_unix(d * 86400);
        assert_eq!(s2, "2026-05-12");
    }

    #[test]
    fn days_between_signed() {
        assert_eq!(days_between("2026-05-12", "2026-05-13"), Some(1));
        assert_eq!(days_between("2026-05-13", "2026-05-12"), Some(-1));
        assert_eq!(days_between("2026-01-01", "2027-01-01"), Some(365));
    }
}
