//! `ven audit` — Comprehensive Dependency Risk Report.
//!
//! Provides a multi-dimensional risk assessment for a ven-managed project's dependencies,
//! going beyond just security vulnerabilities to include licensing, version maturity,
//! compatibility, package health, and ecosystem health.

use anyhow::Result;
use crate::core::audit::{compute_overall_risk_score, risk_level_from_score};
use crate::core::audit_engine::run_audit;
use crate::core::config::find_ven_toml;
use colored::Colorize;
use std::path::Path;

pub fn cmd_audit(json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    
    // Try to find ven.toml
    let cfg = match find_ven_toml(&cwd) {
        Ok(Some(c)) => c,
        Ok(None) => {
            // No ven.toml in cwd; try parent directories
            let mut parent = cwd.parent();
            let mut found = false;
            while let Some(p) = parent {
                if let Ok(Some(c)) = find_ven_toml(p) {
                    found = true;
                    break;
                }
                parent = p.parent();
            }
            if found {
                // Use the found config but report from cwd
                eprintln!(
                    "  {} Using ven.toml from {:?} (current dir: {})",
                    "[WARN]".yellow(),
                    parent.map(|p| p.to_path_buf()).unwrap_or(cwd.clone()),
                    cwd
                );
            } else {
                eprintln!(
                    "  {} No ven.toml found in {} or any parent directory",
                    "[WARN]".yellow(),
                    cwd.display()
                );
            }
            return Ok(());
        }
    };
    
    let report = run_audit(&cwd)?;
    
    if json {
        print_audit_json(&report);
    } else {
        print_audit_terminal(&report);
    }
    
    Ok(())
}

/// Print audit report in JSON format
fn print_audit_json(report: &crate::core::audit::AuditReport) {
    let serde_json::Value::Object(map) = serde_json::to_value(report).unwrap() else {
        eprintln!("Failed to serialize audit report");
        return;
    };
    
    // Add risk level
    let risk_level = risk_level_from_score(report.overall_risk_score);
    map.insert("risk_level".into(), serde_json::Value::String(risk_level.label().to_string()));
    map.insert("risk_score".into(), serde_json::Value::Number(report.overall_risk_score.into()));
    
    println!("{}", serde_json::to_string_pretty(&map).unwrap());
}

/// Print audit report in human-readable terminal format
fn print_audit_terminal(report: &crate::core::audit::AuditReport) {
    let risk_level = risk_level_from_score(report.overall_risk_score);
    let risk_color = risk_level.color();
    
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                    PROJECT DEPENDENCY RISK ASSESSMENT                     ║".bright_black()
    );
    println!(
        "{}",
        format!(
            "║                          Project: {}                              ║",
            report.ecosystem
        )
        .bright_black()
    );
    println!(
        "{}",
        "║                        Directory: ./                                  ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    println!();
    
    // Security Blockers
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                    🛡️ Security (OSV Vulnerabilities)                   ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    if report.security.is_empty() {
        println!(
            "  {} No known vulnerabilities found in {} packages",
            "[OK]".green(),
            report.ecosystem
        );
    } else {
        for finding in &report.security {
            let worst_sev = finding
                .vulnerabilities
                .first()
                .map(|v| &v.severity)
                .unwrap_or(&"NONE".to_string());
            let sev_color = match worst_sev.to_lowercase().as_str() {
                "critical" => "[CRIT]".red().bold(),
                "high" => "[HIGH]".red(),
                "moderate" => "[MED ]".yellow(),
                "low" => "[LOW ]".yellow(),
                _ => "[INFO]".cyan(),
            };
            println!(
                "  {} {}@{} — {} advisory(ies), worst {}",
                sev_color,
                finding.package.bold(),
                finding.version,
                finding.vulnerabilities.len(),
                worst_sev
            );
            for vuln in &finding.vulnerabilities {
                let fix_hint = vuln
                    .fixed_version
                    .as_deref()
                    .map(|f| format!("  → patched in {}", f.bold()))
                    .unwrap_or_default();
                println!(
                    "    - {} [{}]  {}{}",
                    vuln.id.dimmed(),
                    vuln.severity,
                    vuln.summary.as_deref().unwrap_or("(no summary)"),
                    fix_hint
                );
            }
        }
    }
    println!();
    
    // Licensing Concerns
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                    📜 Licensing Concerns                                ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    let copyleft_pkgs: Vec<&LicensingFinding> = report
        .licensing
        .iter()
        .filter(|f| f.copyleft)
        .collect();
    
    if copyleft_pkgs.is_empty() {
        println!(
            "  {} No copyleft license concerns",
            "[OK]".green()
        );
    } else {
        for finding in &copyleft_pkgs {
            println!(
                "  {} {}@{} — copyleft license: {}",
                "[WARN]".yellow(),
                finding.package.bold(),
                finding.version,
                finding.license
            );
        }
    }
    println!();
    
    // Version Maturity
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                  📈 Version Maturity                                    ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    let outdated_pkgs: Vec<&VersionMaturity> = report
        .version_maturity
        .iter()
        .filter(|f| !f.is_latest)
        .collect();
    
    if outdated_pkgs.is_empty() {
        println!(
            "  {} All packages are at latest version",
            "[OK]".green()
        );
    } else {
        for finding in &outdated_pkgs {
            let status = if finding.version == finding.latest_version {
                "[OK]".green().to_string()
            } else {
                "[UPDATE]".yellow().to_string()
            };
            println!(
                "  {} {}@{} (latest: {})",
                status,
                finding.package.bold(),
                finding.version,
                finding.latest_version
            );
        }
    }
    println!();
    
    // Compatibility
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                  ⚠️ Compatibility                                       ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    if report.compatibility.is_empty() {
        println!(
            "  {} No compatibility issues detected",
            "[OK]".green()
        );
    } else {
        for finding in &report.compatibility {
            println!(
                "  {} {}@{} — {}",
                "[WARN]".yellow(),
                finding.package.bold(),
                finding.version,
                finding.conflict_type
            );
        }
    }
    println!();
    
    // Package Health
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                  📦 Package Health                                        ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    let health_concerns: Vec<&PackageHealth> = report
        .package_health
        .iter()
        .filter(|f| f.days_since_publish > 365 || f.repo_activity == "low")
        .collect();
    
    if health_concerns.is_empty() {
        println!(
            "  {} All packages have good health metrics",
            "[OK]".green()
        );
    } else {
        for finding in &health_concerns {
            let age_status = if finding.days_since_publish > 365 {
                "[STALE]".red().to_string()
            } else {
                "[FRESH]".green().to_string()
            };
            let activity_status = if finding.repo_activity == "low" {
                "[LOW ACTIVITY]".yellow().to_string()
            } else {
                "[HEALTHY]".green().to_string()
            };
            println!(
                "  {} {}@{} — {} (age: {}), deps: {}, activity: {}",
                age_status,
                finding.package.bold(),
                finding.version,
                activity_status,
                format_age(finding.days_since_publish),
                finding.dependency_count,
                finding.repo_activity
            );
        }
    }
    println!();
    
    // Ecosystem Health
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                🌐 Ecosystem Health                                        ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    println!(
        "  {} {} packages, {} active maintainers, {} recent releases, popularity {:.1}/10",
        "[INFO]".cyan(),
        report.ecosystem_health.package_count,
        report.ecosystem_health.active_maintainers,
        report.ecosystem_health.recent_releases,
        report.ecosystem_health.popularity_score
    );
    println!();
    
    // Overall Risk Summary
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                    📊 Overall Risk Summary                                ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    let risk_color = risk_level.color();
    println!(
        "  {} Risk Level: {} {:.1}/10",
        risk_color.bold(),
        risk_level.label(),
        report.overall_risk_score
    );
    
    println!(
        "  {} Security: {} findings, Licensing: {} findings",
        "[INFO]".cyan(),
        report.security.len(),
        report.licensing.len()
    );
    println!(
        "  {} Version Maturity: {} findings, Compatibility: {} findings",
        "[INFO]".cyan(),
        report.version_maturity.len(),
        report.compatibility.len()
    );
    println!(
        "  {} Package Health: {} findings, Ecosystem Health: reviewed",
        "[INFO]".cyan(),
        report.package_health.len()
    );
    println!();
    
    // Recommendations
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║                  💡 Recommendations                                       ║".bright_black()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════════════════╝".bright_black()
    );
    
    if report.recommendations.is_empty() {
        println!("  {} No recommendations", "[OK]".green());
    } else {
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().green(), rec);
        }
    }
    println!();
}

/// Format age in days as human-readable string
fn format_age(days: u64) -> String {
    if days < 30 {
        format!("{} days ago", days)
    } else if days < 365 {
        let months = days / 30;
        format!("{} months ago", months)
    } else {
        let years = days / 365;
        format!("{} years ago", years)
    }
}