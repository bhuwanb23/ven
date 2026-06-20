//! `ven check` — combined security (OSV) + EOL health report.

use crate::core::block_on_async;
use crate::core::config::VenConfig;
use crate::core::endoflife::{endoflife_slug_for_runtime_name, EndOfLifeClient, EolReport};
use crate::core::load_config;
use crate::core::osv::{osv_ecosystem_for, OsvClient, OsvPackageReport, OsvQuery, SeverityRank};
use crate::intelligence::graph::RuntimeKind;
use crate::intelligence::ven_lock::VenLockFile;
use anyhow::Result;
use colored::Colorize;
use std::collections::BTreeMap;
use std::path::Path;

pub fn cmd_check(security_only: bool, eol_only: bool, json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    let want_security = security_only || (!security_only && !eol_only);
    let want_eol = eol_only || (!security_only && !eol_only);

    let mut sec_reports: Vec<OsvPackageReport> = Vec::new();
    if want_security {
        sec_reports = run_security(&cwd, &cfg)?;
    }

    let mut eol_reports: Vec<EolReport> = Vec::new();
    if want_eol {
        eol_reports = run_eol(&cfg)?;
    }

    let high_or_worse_cves = sec_reports
        .iter()
        .flat_map(|r| r.vulns.iter())
        .filter(|v| {
            matches!(
                SeverityRank::from_label(&v.severity_label),
                SeverityRank::High | SeverityRank::Critical
            )
        })
        .count();

    let eol_passed = eol_reports.iter().filter(|r| r.eol_passed).count();
    let support_ended = eol_reports.iter().filter(|r| r.support_passed).count();
    let actionable = high_or_worse_cves + eol_passed + support_ended;

    if json {
        let out = serde_json::json!({
            "project": cwd.to_string_lossy(),
            "ran": {
                "security": want_security,
                "eol": want_eol,
            },
            "security": sec_reports,
            "eol": eol_reports,
            "summary": {
                "high_or_critical_cves": high_or_worse_cves,
                "eol_runtimes": eol_passed,
                "support_ended": support_ended,
                "actionable": actionable,
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if want_security {
            print_security_report(&sec_reports);
        }
        if want_eol {
            print_eol_report(&eol_reports);
        }
        println!();
        if actionable == 0 {
            println!("  {} No actionable issues.", "[OK]".green().bold());
        } else {
            println!(
                "  {} {} actionable issue(s) ({} HIGH/CRITICAL CVE, {} runtime past EOL, {} support ended)",
                "[FAIL]".red().bold(),
                actionable,
                high_or_worse_cves,
                eol_passed,
                support_ended
            );
        }
    }

    if actionable > 0 {
        anyhow::bail!("ven check: {} actionable issue(s)", actionable);
    }
    Ok(())
}

fn run_security(cwd: &Path, cfg: &VenConfig) -> Result<Vec<OsvPackageReport>> {
    let kind = primary_runtime_kind(cfg);
    let Some(eco) = osv_ecosystem_for(&kind) else {
        return Ok(Vec::new());
    };
    let pinned = collect_pinned_packages(cwd, cfg, &kind);
    if pinned.is_empty() {
        return Ok(Vec::new());
    }
    let queries: Vec<OsvQuery> = pinned
        .into_iter()
        .map(|(name, version)| OsvQuery::new(eco, name, version))
        .collect();
    let client = OsvClient::new()?;
    let reports = block_on_async(async move { client.query_packages(&queries).await })?;
    reports
}

fn run_eol(cfg: &VenConfig) -> Result<Vec<EolReport>> {
    let mut probes: Vec<(&'static str, String)> = Vec::new();
    for (name, version) in declared_runtimes(cfg) {
        if let Some(slug) = endoflife_slug_for_runtime_name(name) {
            probes.push((slug, version));
        }
    }
    if probes.is_empty() {
        return Ok(Vec::new());
    }
    let client = EndOfLifeClient::new()?;
    let reports = block_on_async(async move {
        let mut out = Vec::new();
        for (slug, version) in probes {
            match client.report(slug, &version).await {
                Ok(r) => out.push(r),
                Err(e) => {
                    eprintln!(
                        "  {} EOL fetch failed for {}@{}: {}",
                        "[WARN]".yellow(),
                        slug,
                        version,
                        e
                    );
                }
            }
        }
        Ok::<_, anyhow::Error>(out)
    })?;
    reports
}

/// `[runtime].<key>` → version pairs that are non-empty.
fn declared_runtimes(cfg: &VenConfig) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    if !cfg.runtime.node.is_empty() {
        out.push(("node", cfg.runtime.node.clone()));
    }
    if !cfg.runtime.bun.is_empty() {
        out.push(("bun", cfg.runtime.bun.clone()));
    }
    if !cfg.runtime.python.is_empty() {
        out.push(("python", cfg.runtime.python.clone()));
    }
    if !cfg.runtime.go.is_empty() {
        out.push(("go", cfg.runtime.go.clone()));
    }
    if !cfg.runtime.rust.is_empty() {
        out.push(("rust", cfg.runtime.rust.clone()));
    }
    if !cfg.runtime.java.is_empty() {
        out.push(("java", cfg.runtime.java.clone()));
    }
    if !cfg.runtime.deno.is_empty() {
        out.push(("deno", cfg.runtime.deno.clone()));
    }
    if !cfg.runtime.ruby.is_empty() {
        out.push(("ruby", cfg.runtime.ruby.clone()));
    }
    out
}

/// Pick the "primary" runtime kind for OSV ecosystem mapping. Mirrors the
/// precedence in `adapter_from_ven_config` but flattened — for security we
/// only run one ecosystem per project (the most ven-aware one).
pub(crate) fn primary_runtime_kind(cfg: &VenConfig) -> RuntimeKind {
    let r = &cfg.runtime;
    if !r.node.is_empty() {
        return RuntimeKind::NpmFamily;
    }
    if !r.bun.is_empty() {
        return RuntimeKind::NpmFamily;
    }
    if !r.python.is_empty() {
        return RuntimeKind::Python;
    }
    if !r.go.is_empty() {
        return RuntimeKind::Go;
    }
    if !r.rust.is_empty() {
        return RuntimeKind::Rust;
    }
    if !r.java.is_empty() {
        return RuntimeKind::Java;
    }
    if !r.ruby.is_empty() {
        return RuntimeKind::Ruby;
    }
    if !r.deno.is_empty() {
        return RuntimeKind::Deno;
    }
    RuntimeKind::Stub
}

/// Pinned (name, version) pairs for the primary ecosystem. Prefers ven.lock
/// when present (full transitive set + integrity), falls back to ven.toml
/// roots only.
pub(crate) fn collect_pinned_packages(
    cwd: &Path,
    cfg: &VenConfig,
    kind: &RuntimeKind,
) -> Vec<(String, String)> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();

    if matches!(kind, RuntimeKind::NpmFamily) {
        let lock_path = cwd.join("ven.lock");
        if lock_path.is_file() {
            if let Ok(lock) = VenLockFile::read_path(&lock_path) {
                for (name, pkg) in lock.packages {
                    out.insert(name, pkg.version);
                }
                if !out.is_empty() {
                    return out.into_iter().collect();
                }
            }
        }
    }

    for (name, spec) in &cfg.packages {
        let version = if spec.is_empty() || spec == "*" || spec == "latest" {
            String::from("0.0.0") // OSV will report nothing for non-existent
                                  // versions; better to skip but we keep
                                  // the entry so the user sees the intent.
        } else {
            spec.trim_start_matches(['^', '~', '=']).to_string()
        };
        if version != "0.0.0" {
            out.insert(name.clone(), version);
        }
    }
    out.into_iter().collect()
}

fn print_security_report(reports: &[OsvPackageReport]) {
    println!();
    println!("{}", "Security (OSV)".bold().cyan());
    if reports.is_empty() {
        println!(
            "  {} No packages to scan (no ven.lock; ven.toml [packages] empty or unpinned).",
            "[INFO]".cyan()
        );
        return;
    }
    let mut counts = (0usize, 0usize, 0usize, 0usize);
    let any_vulns = reports.iter().any(|r| !r.vulns.is_empty());
    if !any_vulns {
        println!(
            "  {} {} package(s) scanned — no advisories.",
            "[OK]".green(),
            reports.len()
        );
        return;
    }
    for r in reports {
        if r.vulns.is_empty() {
            continue;
        }
        let worst = r.worst_severity();
        let label = colorize_sev(worst);
        println!(
            "  {} {}@{}  ({} vuln{})",
            label,
            r.package.bold(),
            r.version,
            r.vulns.len(),
            if r.vulns.len() == 1 { "" } else { "s" }
        );
        for v in &r.vulns {
            let sev = SeverityRank::from_label(&v.severity_label);
            match sev {
                SeverityRank::Critical => counts.3 += 1,
                SeverityRank::High => counts.2 += 1,
                SeverityRank::Moderate => counts.1 += 1,
                SeverityRank::Low => counts.0 += 1,
                _ => {}
            }
            let summary = v.summary.as_deref().unwrap_or("(no summary)");
            let fix_hint = v
                .fixed_version
                .as_deref()
                .map(|f| format!("  fixed in: {}", f.bold()))
                .unwrap_or_default();
            println!(
                "      - {} [{}]  {}{}",
                v.id.dimmed(),
                v.severity_label,
                summary,
                fix_hint
            );
            if !v.cves.is_empty() {
                println!("         CVEs: {}", v.cves.join(", ").dimmed());
            }
            println!("         {}", v.url.dimmed());
        }
    }
    println!();
    println!(
        "  Totals: {} critical, {} high, {} moderate, {} low",
        counts.3, counts.2, counts.1, counts.0
    );
}

fn print_eol_report(reports: &[EolReport]) {
    println!();
    println!("{}", "Runtime end-of-life".bold().cyan());
    if reports.is_empty() {
        println!("  {} No runtimes declared.", "[INFO]".cyan());
        return;
    }
    for r in reports {
        let cycle = r.matched_cycle.as_deref().unwrap_or("?");
        let cache_tag = if r.from_cache {
            " (cached)".dimmed().to_string()
        } else {
            String::new()
        };
        let header = format!(
            "{} {}",
            r.product.bold(),
            format!("{} (cycle {})", r.configured_version, cycle).dimmed()
        );
        if r.eol_passed {
            println!("  {} {}{}", "[EOL]".red(), header, cache_tag);
        } else if r.support_passed {
            println!("  {} {}{}", "[SUPPORT-ENDED]".yellow(), header, cache_tag);
        } else {
            println!("  {} {}{}", "[OK]".green(), header, cache_tag);
        }
        if let Some(latest) = &r.latest {
            println!("      latest: {}", latest);
        }
        if let Some(date) = &r.eol_date {
            let suffix = match r.days_until_eol {
                Some(d) if d < 0 => format!(" — {} days ago", -d),
                Some(d) => format!(" — in {} days", d),
                None => String::new(),
            };
            println!("      eol:    {}{}", date, suffix);
        }
        if let Some(d_until) = r.days_until_support_end {
            if d_until > 0 {
                println!("      support ends in {} days", d_until);
            } else if d_until < 0 {
                println!("      support ended {} days ago", -d_until);
            }
        }
        println!("      {}", r.source_url.dimmed());
    }
}

fn colorize_sev(label: &str) -> colored::ColoredString {
    match label {
        "CRITICAL" => "[CRIT]".red().bold(),
        "HIGH" => "[HIGH]".red(),
        "MODERATE" => "[MED ]".yellow(),
        "LOW" => "[LOW ]".yellow(),
        _ => "[INFO]".cyan(),
    }
}
