//! Pre-install security gate.
//!
//! `ven add` asks "is this package safe?" *before* anything reaches the disk.
//! Every install funnels through [`run_install_gate`], which resolves the version
//! that is about to be installed, queries OSV.dev for known advisories, and
//! turns the result into a [`GateDecision`].
//!
//! # Why this uses OSV and not the npm bulk-advisories API
//!
//! The older [`crate::core::security::SecurityScanner`] hardcodes
//! `registry.npmjs.org/-/npm/v1/security/advisories/bulk`, so it can only ever
//! answer questions about npm. [`crate::core::osv::OsvClient`] covers all nine
//! ecosystems ven supports via [`crate::core::osv::osv_ecosystem_for`], which is
//! the same client `ven check` already uses. One scanner, one cache, consistent
//! answers between `ven check` and `ven add`.
//!
//! # Policy
//!
//! [`SeverityPolicy::default`] blocks on `CRITICAL` and `HIGH` and only warns on
//! `MODERATE` / `LOW`. A gate that fires on every advisory trains people to
//! reflexively override it, which is worse than not having one.
//!
//! # Failure policy
//!
//! A network failure or an unresolvable version is **not** treated as "clean".
//! It degrades to a warning by default so an offline laptop isn't bricked, but
//! [`GatePolicy::fail_closed`] turns it into a block, and `--strict` on the CLI
//! turns it on. Unauditable means "we don't know", never "it's fine".

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use crate::core::osv::{osv_ecosystem_for, OsvClient, OsvPackageReport, OsvQuery, SeverityRank};
use crate::intelligence::graph::RuntimeKind;

/// Severity at (and above) which the gate refuses to proceed unattended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityPolicy {
    /// Warn only; never blocks.
    Advisory,
    /// Block on `HIGH` and `CRITICAL` (default).
    HighAndCritical,
    /// Block on `CRITICAL` only.
    CriticalOnly,
}

impl Default for SeverityPolicy {
    fn default() -> Self {
        SeverityPolicy::HighAndCritical
    }
}

impl SeverityPolicy {
    /// True when a finding of `rank` should stop the install.
    pub fn blocks(self, rank: SeverityRank) -> bool {
        match self {
            SeverityPolicy::Advisory => false,
            SeverityPolicy::CriticalOnly => rank >= SeverityRank::Critical,
            SeverityPolicy::HighAndCritical => rank >= SeverityRank::High,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SeverityPolicy::Advisory => "advisory",
            SeverityPolicy::CriticalOnly => "critical",
            SeverityPolicy::HighAndCritical => "high+critical",
        }
    }
}

/// Knobs the CLI layer owns and the gate honours.
#[derive(Debug, Clone, Copy)]
pub struct GatePolicy {
    pub severity: SeverityPolicy,
    /// Treat "could not audit" as a block instead of a warning.
    pub fail_closed: bool,
    /// Skip the gate entirely (`--ignore-vulns`).
    pub disabled: bool,
}

impl Default for GatePolicy {
    fn default() -> Self {
        GatePolicy {
            severity: SeverityPolicy::default(),
            fail_closed: false,
            disabled: false,
        }
    }
}

/// What the gate decided the caller should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    /// Nothing blocking. Install as requested.
    Proceed,
    /// Install, but with at least one package swapped to its patched version.
    /// Carries `(package, fixed_version)` pairs the caller should apply.
    ProceedWithFixed(Vec<(String, String)>),
    /// Do not install.
    Abort,
}

/// A package the gate could not audit, with the reason.
#[derive(Debug, Clone)]
pub struct UnauditedPackage {
    pub name: String,
    pub version: Option<String>,
    pub reason: String,
}

/// Full outcome of one gate run, suitable for `--json`.
#[derive(Debug, Clone, Default)]
pub struct GateReport {
    pub ecosystem: Option<String>,
    pub blocking: Vec<OsvPackageReport>,
    pub warning: Vec<OsvPackageReport>,
    pub unaudited: Vec<UnauditedPackage>,
    pub decision: Option<String>,
}

impl GateReport {
    pub fn is_clean(&self) -> bool {
        self.blocking.is_empty() && self.unaudited.is_empty()
    }
}

/// A package the caller wants installed, before the gate has resolved versions.
#[derive(Debug, Clone)]
pub struct InstallCandidate {
    pub name: String,
    /// `None` means "whatever the package manager would pick" — the gate
    /// resolves a concrete version so OSV can be queried.
    pub version: Option<String>,
}

impl InstallCandidate {
    pub fn new(name: impl Into<String>, version: Option<String>) -> Self {
        InstallCandidate {
            name: name.into(),
            version,
        }
    }
}

/// Resolve the version that would actually be installed, so the audit queries a
/// real (name, version) pair rather than guessing.
///
/// A pinned version is used as-is. An unpinned one is resolved from the
/// ecosystem's registry "latest" endpoint. Resolution failures come back as
/// `Err` so the caller can record them as unaudited rather than silently
/// skipping the scan.
pub fn resolve_install_version(
    ecosystem: &str,
    name: &str,
    pinned: Option<&str>,
) -> Result<String> {
    if let Some(v) = pinned {
        if !v.is_empty() {
            return Ok(v.to_string());
        }
    }
    let url = latest_endpoint(ecosystem, name)?;
    let client = crate::core::integrity::http_client("ven-add-security-gate")?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| anyhow::anyhow!("could not reach {ecosystem} registry: {e}"))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("{ecosystem} registry returned an error: {e}"))?;
    let body = resp
        .text()
        .map_err(|e| anyhow::anyhow!("could not read {ecosystem} registry response: {e}"))?;
    parse_latest(ecosystem, &body, name)
}

fn latest_endpoint(ecosystem: &str, name: &str) -> Result<String> {
    // Scoped npm packages (`@scope/pkg`) keep the slash when addressing the
    // registry, but the name is otherwise used verbatim in the URL path.
    match ecosystem {
        "npm" => Ok(format!("https://registry.npmjs.org/{name}/latest")),
        "PyPI" => Ok(format!("https://pypi.org/pypi/{name}/json")),
        "crates.io" => Ok(format!("https://crates.io/api/v1/crates/{name}")),
        "Go" => Ok(format!("https://proxy.golang.org/{name}/@latest")),
        "RubyGems" => Ok(format!("https://rubygems.org/api/v1/gems/{name}.json")),
        "Packagist" => Ok(format!("https://repo.packagist.org/p2/{name}.json")),
        // Maven has no cheap "latest" endpoint; coordinates need a real version.
        "Maven" => anyhow::bail!(
            "Maven coordinates have no canonical latest version — pin one to audit it"
        ),
        other => anyhow::bail!("no version-resolution strategy for ecosystem {other:?}"),
    }
}

fn parse_latest(ecosystem: &str, body: &str, name: &str) -> Result<String> {
    let version = match ecosystem {
        "npm" => serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from)),
        "PyPI" => serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("info")
                    .and_then(|i| i.get("version"))
                    .and_then(|s| s.as_str())
                    .map(String::from)
            }),
        "crates.io" => serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("crate")
                    .and_then(|c| c.get("max_stable_version").or_else(|| c.get("max_version")))
                    .and_then(|s| s.as_str())
                    .map(String::from)
            }),
        // Go's @latest endpoint returns a plain version string.
        "Go" => Some(body.trim().trim_start_matches('v').to_string()),
        "RubyGems" => serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("version")
                    .and_then(|version| version.get("number"))
                    .and_then(|s| s.as_str())
                    .map(String::from)
            }),
        "Packagist" => serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("packages")
                    .and_then(|p| p.get(name))
                    .and_then(|a| a.as_array())
                    .and_then(|a| a.first())
                    .and_then(|o| o.get("version"))
                    .and_then(|s| s.as_str())
                    .map(String::from)
            }),
        _ => None,
    };
    match version {
        Some(v) if !v.is_empty() => Ok(v),
        _ => anyhow::bail!(
            "could not determine the version {name:?} would install to; pin a version to audit it"
        ),
    }
}

/// Audit `candidates` against OSV and return the report. Performs no I/O beyond
/// the registry/version lookups and the OSV query itself.
pub fn audit_candidates(
    kind: &RuntimeKind,
    candidates: &[InstallCandidate],
    policy: &GatePolicy,
) -> Result<GateReport> {
    let mut report = GateReport::default();
    let Some(ecosystem) = osv_ecosystem_for(kind) else {
        // Unknown primary runtime — nothing to audit, and nothing to claim.
        return Ok(report);
    };
    report.ecosystem = Some(ecosystem.to_string());

    let mut queries = Vec::new();
    for candidate in candidates {
        match resolve_install_version(ecosystem, &candidate.name, candidate.version.as_deref()) {
            Ok(version) => queries.push(OsvQuery::new(ecosystem, &candidate.name, version)),
            Err(e) => report.unaudited.push(UnauditedPackage {
                name: candidate.name.clone(),
                version: candidate.version.clone(),
                reason: e.to_string(),
            }),
        }
    }

    if queries.is_empty() {
        return Ok(report);
    }

    let client = OsvClient::new()?;
    let reports =
        // `block_on_async` wraps the future's own `Result`, so both layers need `?`
        // (unlike `cli::check`, whose function returns that inner `Result` directly).
        crate::core::block_on_async(async move { client.query_packages(&queries).await })??;
    for r in reports {
        if report_worst_rank(&r).is_some_and(|rank| policy.severity.blocks(rank)) {
            report.blocking.push(r);
        } else if !r.vulns.is_empty() {
            report.warning.push(r);
        }
    }
    Ok(report)
}

/// The highest [`SeverityRank`] present in a report, or `None` when clean.
pub fn report_worst_rank(report: &OsvPackageReport) -> Option<SeverityRank> {
    report
        .vulns
        .iter()
        .map(|v| SeverityRank::from_label(&v.severity_label))
        .max()
        .filter(|rank| *rank > SeverityRank::None)
}

/// Best patched version advertised across a report's advisories, if any.
pub fn best_fixed_version(report: &OsvPackageReport) -> Option<String> {
    report
        .vulns
        .iter()
        .filter_map(|v| v.fixed_version.clone())
        .max_by(|a, b| compare_versions(a, b))
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    parse(a).cmp(&parse(b))
}

/// Render the human-readable audit block shown before installing.
pub fn print_report(report: &GateReport) {
    println!("\n  {}", "Security Audit".bold().cyan());
    let eco = report.ecosystem.as_deref().unwrap_or("unknown ecosystem");
    if report.blocking.is_empty() && report.warning.is_empty() && report.unaudited.is_empty() {
        println!(
            "  {} No known vulnerabilities found in {eco} packages",
            "[OK]".green()
        );
        return;
    }

    if !report.blocking.is_empty() {
        println!(
            "  {} {} blocking advisories in {eco}:",
            "🚨".red().bold(),
            report.blocking.len()
        );
    }
    for r in &report.blocking {
        print_one(r, true);
    }
    if !report.warning.is_empty() {
        println!(
            "  {} {} advisory(ies) below the blocking threshold:",
            "⚠".yellow(),
            report.warning.len()
        );
    }
    for r in &report.warning {
        print_one(r, false);
    }
    if !report.unaudited.is_empty() {
        println!(
            "  {} {} package(s) could NOT be audited:",
            "⚠".yellow().bold(),
            report.unaudited.len()
        );
        for u in &report.unaudited {
            let version = u.version.as_deref().unwrap_or("latest");
            println!("    {} {u}@{version} — {}", "[?]".yellow(), u.reason);
        }
    }
}

fn print_one(report: &OsvPackageReport, blocking: bool) {
    let mark = if blocking {
        "[VULN]".red()
    } else {
        "[WARN]".yellow()
    };
    println!(
        "  {} {}@{} — {} advisory(ies), worst {}",
        mark,
        report.package,
        report.version,
        report.vulns.len(),
        report.worst_severity()
    );
    for vuln in &report.vulns {
        println!("      {}", vuln.id.bold());
        if let Some(summary) = &vuln.summary {
            println!("        {}", summary);
        }
        if let Some(fixed) = &vuln.fixed_version {
            println!("        {} patched in {fixed}", "→".green());
        }
        if !vuln.url.is_empty() {
            println!("        {}", vuln.url.dimmed());
        }
    }
}

/// Best patched versions per package for the reports that block.
pub fn fixed_versions_for(report: &GateReport) -> Vec<(String, String)> {
    report
        .blocking
        .iter()
        .filter_map(|r| best_fixed_version(r).map(|fixed| (r.package.clone(), fixed)))
        .collect()
}

/// Interactive decision. Returns `None` when there is no TTY to prompt on.
pub fn prompt_decision(
    report: &GateReport,
    policy: &GatePolicy,
    yes: bool,
    json: bool,
) -> Result<Option<GateDecision>> {
    let has_issue =
        !report.blocking.is_empty() || (policy.fail_closed && !report.unaudited.is_empty());

    if !has_issue {
        return Ok(Some(GateDecision::Proceed));
    }

    // `--yes` and `--json` are non-interactive by definition: never block a
    // pipeline behind a prompt nobody can answer.
    if yes || json {
        return Ok(None);
    }

    if !is_interactive() {
        return Ok(None);
    }

    use dialoguer::{theme::ColorfulTheme, Select};
    let theme = ColorfulTheme::default();

    let fixes = fixed_versions_for(report);
    let mut items: Vec<String> = Vec::new();
    if !fixes.is_empty() {
        let preview = fixes
            .iter()
            .map(|(p, v)| format!("{p}@{v}"))
            .collect::<Vec<_>>()
            .join(", ");
        items.push(format!("Install anyway (install vulnerable version(s))"));
        items.push(format!("Use patched version(s) instead ({preview})"));
    } else {
        items.push("Install anyway (install vulnerable version(s))".to_string());
    }
    items.push("Cancel — don't install anything".to_string());

    let choice = match Select::with_theme(&theme)
        .with_prompt("ven found security issues. How should it proceed?")
        .items(&items)
        .default(0)
        .interact()
    {
        Ok(choice) => choice,
        // A prompt that cannot be answered (no TTY, piped stdin, closed handle)
        // must not turn into an opaque error. Treat it as non-interactive so
        // the caller applies the documented non-TTY policy and exit code.
        Err(e) => {
            eprintln!(
                "  {} Could not prompt ({e}); falling back to non-interactive policy.",
                "[WARN]".yellow()
            );
            return Ok(None);
        }
    };

    Ok(Some(match choice {
        0 => GateDecision::Proceed,
        1 if !fixes.is_empty() => GateDecision::ProceedWithFixed(fixes),
        _ => GateDecision::Abort,
    }))
}

/// Best-effort TTY probe. `dialoguer` returns an error rather than hanging when
/// stdin is redirected, but checking up front lets us return a clean
/// non-interactive result instead.
fn is_interactive() -> bool {
    #[cfg(unix)]
    {
        // /dev/tty openable => a controlling terminal exists.
        std::fs::OpenOptions::new()
            .read(true)
            .open("/dev/tty")
            .is_ok()
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Full gate: audit, report, decide. Returns the action the caller must take.
pub fn run_install_gate(
    kind: &RuntimeKind,
    candidates: &[InstallCandidate],
    policy: &GatePolicy,
    yes: bool,
    json: bool,
) -> Result<(GateDecision, GateReport)> {
    if policy.disabled {
        let mut report = GateReport::default();
        report.decision = Some("skipped".to_string());
        return Ok((GateDecision::Proceed, report));
    }

    let mut report = audit_candidates(kind, candidates, policy)?;

    print_report(&report);

    let decision = match prompt_decision(&report, policy, yes, json)? {
        Some(d) => d,
        None => {
            let blocking = !report.blocking.is_empty();
            let unaudited_blocks = policy.fail_closed && !report.unaudited.is_empty();
            if blocking || unaudited_blocks {
                GateDecision::Abort
            } else {
                GateDecision::Proceed
            }
        }
    };

    report.decision = Some(
        match &decision {
            GateDecision::Proceed => "proceed",
            GateDecision::ProceedWithFixed(_) => "proceed_with_fixed_versions",
            GateDecision::Abort => "abort",
        }
        .to_string(),
    );
    Ok((decision, report))
}

/// Exit code contract for the security gate (mirrors `docs/security-model.md`):
/// `2` means "refused because of a security finding", which is distinct from `1`
/// (generic error) so CI can tell a vulnerability block from a crash.
pub const EXIT_SECURITY_BLOCKED: i32 = 2;

pub fn exit_code_for(decision: &GateDecision) -> i32 {
    match decision {
        GateDecision::Abort => EXIT_SECURITY_BLOCKED,
        _ => 0,
    }
}

/// Machine-readable payload for `ven add --json`.
pub fn report_to_json(report: &GateReport, decision: &GateDecision) -> serde_json::Value {
    serde_json::json!({
        "ecosystem": report.ecosystem,
        "decision": report.decision,
        "blocking": report.blocking,
        "warnings": report.warning,
        "unaudited": report.unaudited.iter().map(|u| serde_json::json!({
            "package": u.name,
            "version": u.version,
            "reason": u.reason,
        })).collect::<Vec<_>>(),
        "patched_versions": match decision {
            GateDecision::ProceedWithFixed(fixes) => fixes
                .iter()
                .map(|(p, v)| serde_json::json!({ "package": p, "version": v }))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        },
        "exit_code": exit_code_for(decision),
    })
}

/// Convenience: build candidates from `ven add` style specs (`name` or
/// `name@version`). Scoped npm packages (`@scope/pkg`) are preserved.
pub fn candidates_from_specs(specs: &[String]) -> Vec<InstallCandidate> {
    specs
        .iter()
        .map(|spec| {
            // `@scope/pkg` starts with '@' and must not be split there.
            let at = spec.rfind('@').filter(|i| *i > 0);
            match at {
                Some(i) => InstallCandidate::new(&spec[..i], Some(spec[i + 1..].to_string())),
                None => InstallCandidate::new(spec.as_str(), None),
            }
        })
        .collect()
}

/// Map a `ven.toml` package map into candidates (used when re-auditing an
/// already-declared dependency set).
pub fn candidates_from_map(map: &HashMap<String, String>) -> Vec<InstallCandidate> {
    map.iter()
        .map(|(k, v)| InstallCandidate::new(k, Some(v.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_blocks_high_and_critical_by_default() {
        let p = SeverityPolicy::default();
        assert!(!p.blocks(SeverityRank::None));
        assert!(!p.blocks(SeverityRank::Low));
        assert!(!p.blocks(SeverityRank::Moderate));
        assert!(p.blocks(SeverityRank::High));
        assert!(p.blocks(SeverityRank::Critical));
    }

    #[test]
    fn critical_only_policy_ignores_high() {
        let p = SeverityPolicy::CriticalOnly;
        assert!(!p.blocks(SeverityRank::High));
        assert!(p.blocks(SeverityRank::Critical));
    }

    #[test]
    fn advisory_policy_never_blocks() {
        let p = SeverityPolicy::Advisory;
        assert!(!p.blocks(SeverityRank::Critical));
        assert!(!p.blocks(SeverityRank::High));
    }

    #[test]
    fn worst_rank_ignores_clean_reports() {
        let report = OsvPackageReport {
            package: "ok".into(),
            version: "1.0.0".into(),
            ..Default::default()
        };
        assert_eq!(report_worst_rank(&report), None);
    }

    #[test]
    fn worst_rank_picks_the_most_severe() {
        let report = OsvPackageReport {
            package: "mixed".into(),
            version: "1.0.0".into(),
            vulns: vec![
                crate::core::osv::OsvVuln {
                    id: "GHSA-low".into(),
                    severity_label: "LOW".into(),
                    ..Default::default()
                },
                crate::core::osv::OsvVuln {
                    id: "GHSA-critical".into(),
                    severity_label: "CRITICAL".into(),
                    ..Default::default()
                },
                crate::core::osv::OsvVuln {
                    id: "GHSA-moderate".into(),
                    severity_label: "MODERATE".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(report_worst_rank(&report), Some(SeverityRank::Critical));
    }

    #[test]
    fn best_fixed_version_prefers_the_highest() {
        let report = OsvPackageReport {
            package: "lodash".into(),
            version: "4.17.20".into(),
            vulns: vec![
                crate::core::osv::OsvVuln {
                    id: "A".into(),
                    fixed_version: Some("4.17.15".into()),
                    ..Default::default()
                },
                crate::core::osv::OsvVuln {
                    id: "B".into(),
                    fixed_version: Some("4.17.21".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(best_fixed_version(&report).as_deref(), Some("4.17.21"));
    }

    #[test]
    fn best_fixed_version_is_none_without_advisories() {
        let report = OsvPackageReport::default();
        assert_eq!(best_fixed_version(&report), None);
    }

    #[test]
    fn specs_split_name_and_pinned_version() {
        let specs = vec!["lodash".to_string(), "express@4.18.2".to_string()];
        let c = candidates_from_specs(&specs);
        assert_eq!(c[0].name, "lodash");
        assert_eq!(c[0].version, None);
        assert_eq!(c[1].name, "express");
        assert_eq!(c[1].version.as_deref(), Some("4.18.2"));
    }

    #[test]
    fn scoped_npm_specs_are_not_split_on_leading_at() {
        let specs = vec!["@types/node@20.11.0".to_string()];
        let c = candidates_from_specs(&specs);
        assert_eq!(c[0].name, "@types/node");
        assert_eq!(c[0].version.as_deref(), Some("20.11.0"));
    }

    #[test]
    fn unpinned_scoped_spec_keeps_the_whole_name() {
        let specs = vec!["@types/node".to_string()];
        let c = candidates_from_specs(&specs);
        assert_eq!(c[0].name, "@types/node");
        assert_eq!(c[0].version, None);
    }

    #[test]
    fn exit_code_is_two_only_for_abort() {
        assert_eq!(exit_code_for(&GateDecision::Proceed), 0);
        assert_eq!(
            exit_code_for(&GateDecision::ProceedWithFixed(vec![(
                "a".into(),
                "1.0.0".into()
            )])),
            0
        );
        assert_eq!(exit_code_for(&GateDecision::Abort), 2);
    }

    #[test]
    fn disabled_policy_short_circuits() {
        let policy = GatePolicy {
            disabled: true,
            ..Default::default()
        };
        let (decision, report) =
            run_install_gate(&RuntimeKind::Python, &[], &policy, true, true).unwrap();
        assert_eq!(decision, GateDecision::Proceed);
        assert_eq!(report.decision.as_deref(), Some("skipped"));
    }

    #[test]
    fn stub_runtime_produces_no_ecosystem_and_no_queries() {
        // Must not attempt network I/O for an unknown runtime.
        let report = audit_candidates(&RuntimeKind::Stub, &[], &GatePolicy::default()).unwrap();
        assert_eq!(report.ecosystem, None);
        assert!(report.blocking.is_empty());
    }

    #[test]
    fn json_payload_includes_decision_and_exit_code() {
        let report = GateReport {
            ecosystem: Some("npm".into()),
            decision: Some("abort".into()),
            ..Default::default()
        };
        let json = report_to_json(&report, &GateDecision::Abort);
        assert_eq!(json["decision"], "abort");
        assert_eq!(json["exit_code"], 2);
        assert_eq!(json["ecosystem"], "npm");
    }

    #[test]
    fn version_comparison_is_numeric_not_lexicographic() {
        // 4.17.21 > 4.9.0 numerically, but lexicographically "4.9.0" > "4.17.21".
        use std::cmp::Ordering;
        assert_eq!(compare_versions("4.17.21", "4.9.0"), Ordering::Greater);
    }
}
