//! Per-ecosystem documentation fetcher with terminal rendering.
//!
//! Resolves the version pin (`ven.lock` → `ven.toml [packages]` → installed
//! manifest), fetches the canonical docs payload, and either renders it in
//! the terminal (`termimad`) or opens the canonical URL in the default
//! browser (`webbrowser`). Diff mode runs `similar::TextDiff` over the
//! fetched READMEs at the two requested versions.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::config::VenConfig;
use crate::intelligence::graph::RuntimeKind;
use crate::intelligence::ven_lock::VenLockFile;

pub const DOC_CACHE_TTL_SECS: i64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone)]
pub struct DocRequest {
    pub kind: RuntimeKind,
    pub package: String,
    pub version: String,
    pub browser: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DocOutcome {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    /// Canonical docs URL (always set when known).
    pub url: Option<String>,
    /// Markdown / plain-text rendering for the terminal. `None` for
    /// `--browser` mode and for ecosystems that only emit a URL.
    pub rendered: Option<String>,
    /// Diff result (when called from [`diff_versions`]).
    pub diff: Option<String>,
    /// Free-form one-liner shown above the body.
    pub note: Option<String>,
    pub from_cache: bool,
    pub opened_in_browser: bool,
}

/// Resolve the version we'll show docs for. Order:
///  1. `ven.lock` packages
///  2. `ven.toml [packages]` (strip `^`/`~` prefixes)
///  3. installed manifests (`node_modules/<pkg>/package.json`, …) — best-effort
pub fn resolve_pinned_version(
    cwd: &Path,
    cfg: &VenConfig,
    kind: &RuntimeKind,
    package: &str,
) -> Result<Option<String>> {
    if matches!(kind, RuntimeKind::NpmFamily) {
        let lock_path = cwd.join("ven.lock");
        if lock_path.is_file() {
            if let Ok(lock) = VenLockFile::read_path(&lock_path) {
                if let Some(p) = lock.packages.get(package) {
                    return Ok(Some(p.version.clone()));
                }
            }
        }
    }
    if let Some(spec) = cfg.packages.get(package) {
        let s = spec.trim_start_matches(['^', '~', '=']).trim();
        if !s.is_empty() && s != "*" && s != "latest" {
            return Ok(Some(s.to_string()));
        }
    }

    // Installed-manifest probe (npm only — others vary too much).
    if matches!(kind, RuntimeKind::NpmFamily) {
        let pkg_json = cwd.join("node_modules").join(package).join("package.json");
        if let Ok(body) = fs::read_to_string(&pkg_json) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(s) = v.get("version").and_then(|x| x.as_str()) {
                    return Ok(Some(s.to_string()));
                }
            }
        }
    }
    Ok(None)
}

pub fn render_doc(req: &DocRequest) -> Result<DocOutcome> {
    let mut outcome = DocOutcome {
        package: req.package.clone(),
        version: req.version.clone(),
        ecosystem: ecosystem_name(&req.kind).to_string(),
        ..Default::default()
    };
    let url = canonical_url(&req.kind, &req.package, &req.version);
    outcome.url = url.clone();

    if req.browser {
        if let Some(u) = url.as_deref() {
            if std::env::var("VEN_BROWSER_DRY_RUN").is_ok() {
                outcome.note = Some(format!("(dry-run) would open {}", u));
            } else {
                let _ = webbrowser::open(u);
                outcome.opened_in_browser = true;
            }
        } else {
            outcome.note = Some("No canonical URL for this ecosystem.".to_string());
        }
        return Ok(outcome);
    }

    let cache = DocCache::open()?;
    if let Some((body, source_url)) = cache.get(&outcome.ecosystem, &req.package, &req.version)? {
        outcome.from_cache = true;
        outcome.url = outcome.url.or(Some(source_url));
        outcome.rendered = Some(render_for_terminal(&body));
        return Ok(outcome);
    }

    match fetch_doc_body(&req.kind, &req.package, &req.version) {
        Ok((body, source_url)) => {
            let _ = cache.put(
                &outcome.ecosystem,
                &req.package,
                &req.version,
                &body,
                &source_url,
            );
            outcome.url = outcome.url.or(Some(source_url));
            outcome.rendered = Some(render_for_terminal(&body));
        }
        Err(e) => {
            outcome.note = Some(format!(
                "Failed to fetch docs ({}). Try `--browser` to open the canonical URL.",
                e
            ));
        }
    }
    Ok(outcome)
}

pub fn diff_versions(kind: &RuntimeKind, package: &str, v1: &str, v2: &str) -> Result<DocOutcome> {
    let cache = DocCache::open()?;
    let eco = ecosystem_name(kind).to_string();

    let body_a = fetch_or_cache(&cache, kind, &eco, package, v1)?;
    let body_b = fetch_or_cache(&cache, kind, &eco, package, v2)?;

    let diff = similar::TextDiff::from_lines(&body_a, &body_b);
    let mut out = String::new();
    out.push_str(&format!("--- {}@{}\n", package, v1));
    out.push_str(&format!("+++ {}@{}\n", package, v2));
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        out.push_str(sign);
        out.push_str(change.value());
    }

    Ok(DocOutcome {
        package: package.to_string(),
        version: format!("{}…{}", v1, v2),
        ecosystem: eco,
        url: canonical_url(kind, package, v2),
        rendered: None,
        diff: Some(out),
        note: Some(format!(
            "Showing line diff of READMEs between {} and {}",
            v1, v2
        )),
        from_cache: false,
        opened_in_browser: false,
    })
}

fn fetch_or_cache(
    cache: &DocCache,
    kind: &RuntimeKind,
    eco: &str,
    package: &str,
    version: &str,
) -> Result<String> {
    if let Some((body, _)) = cache.get(eco, package, version)? {
        return Ok(body);
    }
    let (body, source_url) = fetch_doc_body(kind, package, version)?;
    let _ = cache.put(eco, package, version, &body, &source_url);
    Ok(body)
}

// ── per-ecosystem fetch ─────────────────────────────────────────────────────

fn fetch_doc_body(kind: &RuntimeKind, package: &str, version: &str) -> Result<(String, String)> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("ven/0.1.0 (+https://github.com/yourorg/ven)")
        .build()?;

    match kind {
        RuntimeKind::NpmFamily => {
            let url = format!("https://registry.npmjs.org/{}/{}", package, version);
            let resp = client.get(&url).send()?;
            if !resp.status().is_success() {
                anyhow::bail!("npm registry returned {}", resp.status());
            }
            let v: serde_json::Value = resp.json()?;
            let body = v
                .get("readme")
                .and_then(|s| s.as_str())
                .map(String::from)
                .unwrap_or_else(|| {
                    v.get("description")
                        .and_then(|s| s.as_str())
                        .map(String::from)
                        .unwrap_or_else(|| format!("(no README for {}@{})", package, version))
                });
            let canonical = format!("https://www.npmjs.com/package/{}/v/{}", package, version);
            Ok((body, canonical))
        }
        RuntimeKind::Python => {
            let url = format!("https://pypi.org/pypi/{}/{}/json", package, version);
            let resp = client.get(&url).send()?;
            if !resp.status().is_success() {
                anyhow::bail!("PyPI returned {}", resp.status());
            }
            let v: serde_json::Value = resp.json()?;
            let info = v.get("info").cloned().unwrap_or_default();
            let body = info
                .get("description")
                .and_then(|s| s.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("(no description for {}@{})", package, version));
            let canonical = format!("https://pypi.org/project/{}/{}/", package, version);
            Ok((body, canonical))
        }
        RuntimeKind::Rust => {
            // docs.rs returns HTML; we punt on rendering and just point at the URL.
            let canonical = format!("https://docs.rs/{}/{}/{}/", package, version, package);
            Ok((
                format!(
                    "Rust docs for `{}@{}` are HTML — open in browser:\n\n  {}\n",
                    package, version, canonical
                ),
                canonical,
            ))
        }
        RuntimeKind::Go => {
            let canonical = format!("https://pkg.go.dev/{}@{}", package, version);
            Ok((
                format!(
                    "Go docs for `{}@{}` are HTML — open in browser:\n\n  {}\n",
                    package, version, canonical
                ),
                canonical,
            ))
        }
        RuntimeKind::Java => {
            // Maven coordinates are `groupId:artifactId`. `package` may be
            // either, so we just point at the canonical javadoc.io URL.
            let canonical = if package.contains(':') {
                let mut it = package.splitn(2, ':');
                let g = it.next().unwrap_or("");
                let a = it.next().unwrap_or("");
                format!("https://javadoc.io/doc/{}/{}/{}", g, a, version)
            } else {
                format!("https://javadoc.io/doc/{}/{}", package, version)
            };
            Ok((
                format!(
                    "Java docs for `{}@{}` are HTML — open in browser:\n\n  {}\n",
                    package, version, canonical
                ),
                canonical,
            ))
        }
        RuntimeKind::Ruby => {
            let url = format!("https://rubygems.org/api/v1/gems/{}.json", package);
            let resp = client.get(&url).send()?;
            if !resp.status().is_success() {
                anyhow::bail!("rubygems.org returned {}", resp.status());
            }
            let v: serde_json::Value = resp.json()?;
            let body = v
                .get("info")
                .and_then(|s| s.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("(no info for {})", package));
            let canonical = v
                .get("documentation_uri")
                .and_then(|s| s.as_str())
                .map(String::from)
                .unwrap_or_else(|| {
                    format!("https://rubygems.org/gems/{}/versions/{}", package, version)
                });
            Ok((body, canonical))
        }
        RuntimeKind::Deno => {
            let canonical = if let Some(rest) = package.strip_prefix("npm:") {
                format!("https://www.npmjs.com/package/{}", rest)
            } else if let Some(rest) = package.strip_prefix("jsr:") {
                format!("https://jsr.io/{}", rest)
            } else if package.starts_with("https://") || package.starts_with("http://") {
                package.to_string()
            } else {
                format!("https://deno.land/x/{}@{}", package, version)
            };
            Ok((
                format!(
                    "Deno docs for `{}@{}` — open in browser:\n\n  {}\n",
                    package, version, canonical
                ),
                canonical,
            ))
        }
        RuntimeKind::Stub => {
            anyhow::bail!("No primary runtime configured; cannot resolve docs.")
        }
    }
}

fn canonical_url(kind: &RuntimeKind, package: &str, version: &str) -> Option<String> {
    Some(match kind {
        RuntimeKind::NpmFamily => {
            format!("https://www.npmjs.com/package/{}/v/{}", package, version)
        }
        RuntimeKind::Python => format!("https://pypi.org/project/{}/{}/", package, version),
        RuntimeKind::Rust => format!("https://docs.rs/{}/{}/{}/", package, version, package),
        RuntimeKind::Go => format!("https://pkg.go.dev/{}@{}", package, version),
        RuntimeKind::Java => {
            if package.contains(':') {
                let mut it = package.splitn(2, ':');
                let g = it.next().unwrap_or("");
                let a = it.next().unwrap_or("");
                format!("https://javadoc.io/doc/{}/{}/{}", g, a, version)
            } else {
                format!("https://javadoc.io/doc/{}/{}", package, version)
            }
        }
        RuntimeKind::Ruby => {
            format!("https://rubygems.org/gems/{}/versions/{}", package, version)
        }
        RuntimeKind::Deno => {
            if let Some(rest) = package.strip_prefix("npm:") {
                format!("https://www.npmjs.com/package/{}", rest)
            } else if let Some(rest) = package.strip_prefix("jsr:") {
                format!("https://jsr.io/{}", rest)
            } else {
                format!("https://deno.land/x/{}@{}", package, version)
            }
        }
        RuntimeKind::Stub => return None,
    })
}

fn ecosystem_name(kind: &RuntimeKind) -> &'static str {
    match kind {
        RuntimeKind::NpmFamily => "npm",
        RuntimeKind::Python => "pypi",
        RuntimeKind::Go => "go",
        RuntimeKind::Rust => "crates.io",
        RuntimeKind::Java => "maven",
        RuntimeKind::Ruby => "rubygems",
        RuntimeKind::Deno => "deno",
        RuntimeKind::Stub => "unknown",
    }
}

/// Render a markdown body for the terminal. Falls back to raw text when not
/// a TTY (CI, pipes) — keeps `--json | jq` and CI logs clean.
pub fn render_for_terminal(markdown: &str) -> String {
    let is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
    if !is_tty {
        return markdown.to_string();
    }
    let skin = termimad::MadSkin::default();
    skin.term_text(markdown).to_string()
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

struct DocCache {
    conn: Connection,
}

impl DocCache {
    fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS doc_cache(
                ecosystem  TEXT NOT NULL,
                package    TEXT NOT NULL,
                version    TEXT NOT NULL,
                fetched_at INTEGER NOT NULL,
                source_url TEXT NOT NULL,
                body       TEXT NOT NULL,
                PRIMARY KEY(ecosystem, package, version)
            );",
        )?;
        Ok(Self { conn })
    }

    fn get(
        &self,
        ecosystem: &str,
        package: &str,
        version: &str,
    ) -> Result<Option<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT body, source_url, fetched_at FROM doc_cache WHERE ecosystem=?1 AND package=?2 AND version=?3",
        )?;
        let row: Option<(String, String, i64)> = stmt
            .query_row(params![ecosystem, package, version], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .optional()?;
        match row {
            Some((body, source_url, ts)) if now_secs().saturating_sub(ts) <= DOC_CACHE_TTL_SECS => {
                Ok(Some((body, source_url)))
            }
            _ => Ok(None),
        }
    }

    fn put(
        &self,
        ecosystem: &str,
        package: &str,
        version: &str,
        body: &str,
        source_url: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO doc_cache(ecosystem, package, version, fetched_at, source_url, body)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(ecosystem, package, version) DO UPDATE SET
               fetched_at = excluded.fetched_at,
               source_url = excluded.source_url,
               body       = excluded.body",
            params![ecosystem, package, version, now_secs(), source_url, body],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_names_cover_all_runtimes() {
        for kind in [
            RuntimeKind::NpmFamily,
            RuntimeKind::Python,
            RuntimeKind::Go,
            RuntimeKind::Rust,
            RuntimeKind::Java,
            RuntimeKind::Ruby,
            RuntimeKind::Deno,
        ] {
            let name = ecosystem_name(&kind);
            assert_ne!(name, "unknown", "missing ecosystem name for {:?}", kind);
        }
    }

    #[test]
    fn canonical_url_npm() {
        let url = canonical_url(&RuntimeKind::NpmFamily, "express", "4.18.2").unwrap();
        assert_eq!(url, "https://www.npmjs.com/package/express/v/4.18.2");
    }

    #[test]
    fn canonical_url_java_handles_group_id() {
        let url = canonical_url(
            &RuntimeKind::Java,
            "org.springframework:spring-core",
            "6.0.0",
        )
        .unwrap();
        assert!(url.contains("/org.springframework/spring-core/"));
    }

    #[test]
    fn resolve_pinned_prefers_lock() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        std::fs::write(
            cwd.join("ven.lock"),
            r#"{
                "lock_format_version": 2,
                "ecosystem": "npm",
                "runtime_kind": "NpmFamily",
                "runtime_version": "20",
                "roots": ["express"],
                "packages": {"express": {"version": "4.18.2"}},
                "edges": []
            }"#,
        )
        .unwrap();
        let mut cfg = VenConfig::default();
        cfg.packages.insert("express".into(), "^4.18.0".into());
        let v = resolve_pinned_version(cwd, &cfg, &RuntimeKind::NpmFamily, "express")
            .unwrap()
            .unwrap();
        assert_eq!(v, "4.18.2");
    }

    #[test]
    fn resolve_pinned_falls_back_to_ven_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        let mut cfg = VenConfig::default();
        cfg.packages.insert("express".into(), "^4.18.0".into());
        let v = resolve_pinned_version(cwd, &cfg, &RuntimeKind::NpmFamily, "express")
            .unwrap()
            .unwrap();
        assert_eq!(v, "4.18.0");
    }
}
