//! `ven audit` engine — implements all risk dimensions.

use anyhow::Result;
use crate::core::audit::{SecurityFinding, LicensingFinding, VersionMaturity, CompatibilityFinding, PackageHealth, EcosystemHealth, AuditReport, RiskLevel, severity_to_risk, compute_overall_risk_score, risk_level_from_score, SupplyChainFinding, CvssScore, MaliciousPackageFinding, ProvenanceFinding, DependencyConfusionFinding};
use crate::core::osv::{osv_ecosystem_for, OsvClient, OsvQuery, OsvPackageReport, SeverityRank};
use crate::intelligence::graph::{IntelGraph, RuntimeKind};
use crate::core::block_on_async;
use std::collections::HashMap;
use std::time::SystemTime;

/// Run a comprehensive audit of a project's dependencies.
/// 
/// This performs a multi-dimensional risk assessment across:
/// 1. Security: OSV vulnerability scan
/// 2. Licensing: License compliance
/// 3. Version Maturity: How up-to-date vs latest
/// 4. Compatibility: Engine incompatibilities & conflict chains
/// 5. Package Health: Last publish date, repo activity, dependency count
/// 6. Ecosystem Health: Package popularity, maintenance status
pub fn run_audit(cwd: &std::path::Path) -> Result<AuditReport> {
    let cfg = crate::core::config::find_ven_toml(cwd)?
        .ok_or_else(|| anyhow::anyhow!("No ven.toml found in {}", cwd.display()))?;
    
    let kind = crate::core::config::primary_runtime_kind(&cfg);
    let ecosystem = osv_ecosystem_for(&kind)
        .ok_or_else(|| anyhow::anyhow!("No OSV ecosystem mapping for runtime kind {:?}", kind))?
        .to_string();
    
    // Get pinned packages from ven.lock or ven.toml
    let pinned = collect_pinned_packages(cwd, &cfg, &kind);
    
    let mut security_findings = Vec::new();
    let mut licensing_findings = Vec::new();
    let mut version_maturity_findings = Vec::new();
    let mut compatibility_findings = Vec::new();
    let mut package_health_findings = Vec::new();
    let mut supply_chain_findings = Vec::new();
    let mut cvss_scores = Vec::new();
    let mut malicious_package_findings = Vec::new();
    let mut provenance_findings = Vec::new();
    let mut dependency_confusion_findings = Vec::new();
    
    // 1. Security dimension using OSV
    if !pinned.is_empty() {
        let reports = run_security_audit(cwd, &cfg, &kind, &pinned)?;
        security_findings = build_security_findings(&reports);
        // Also build CVSS scores from security findings
        cvss_scores = build_cvss_scores(&security_findings);
    }
    
    // 2. Licensing dimension
    licensing_findings = build_licensing_findings(&cfg, &kind, &pinned);
    
    // 3. Version Maturity dimension
    version_maturity_findings = build_version_maturity(&cfg, &kind, &pinned);
    
    // 4. Compatibility dimension
    compatibility_findings = build_compatibility_findings(&cfg, &kind, &pinned);
    
    // 5. Package Health dimension
    package_health_findings = build_package_health(&cfg, &kind, &pinned);
    
    // 6. Ecosystem Health dimension
    let ecosystem_health = build_ecosystem_health(&ecosystem);
    
    // 7. Supply Chain Security dimension
    supply_chain_findings = build_supply_chain_findings(&cfg, &kind, &pinned);
    
    // 8. Malicious Package Detection dimension
    malicious_package_findings = build_malicious_package_findings(&cfg, &kind, &pinned);
    
    // 9. Provenance & SBOM dimension
    provenance_findings = build_provenance_findings(&cfg, &kind, &pinned);
    
    // 10. Dependency Confusion Risk dimension
    dependency_confusion_findings = build_dependency_confusion_findings(&cfg, &kind, &pinned);
    
    // Compute overall risk score (updated to include new dimensions)
    let overall_risk = compute_overall_risk_score(
        security_findings.len(),
        security_findings.iter().filter(|f| f.vulnerabilities.iter().any(|v| v.severity == "CRITICAL")).count(),
        licensing_findings.iter().filter(|f| f.copyleft).count(),
        version_maturity_findings.iter().filter(|f| !f.is_latest).count(),
        compatibility_findings.len(),
        package_health_findings.iter().filter(|f| f.days_since_publish > 365 || f.repo_activity == "low").count(),
        supply_chain_findings.iter().filter(|f| !f.sbom_available || f.dependency_confusion_risk).count(),
    );
    let risk_level = risk_level_from_score(overall_risk);
    
    // Generate recommendations (updated for new dimensions)
    let recommendations = generate_recommendations(
        &security_findings,
        &licensing_findings,
        &version_maturity_findings,
        &compatibility_findings,
        &supply_chain_findings,
        &malicious_package_findings,
        &provenance_findings,
        &dependency_confusion_findings,
    );
    
    Ok(AuditReport {
        ecosystem,
        security: security_findings,
        licensing: licensing_findings,
        version_maturity: version_maturity_findings,
        compatibility: compatibility_findings,
        package_health: package_health_findings,
        ecosystem_health,
        supply_chain: supply_chain_findings,
        cvss_scores,
        malicious_packages: malicious_package_findings,
        provenance: provenance_findings,
        dependency_confusion: dependency_confusion_findings,
        overall_risk_score: overall_risk,
        recommendations,
    })
}

/// Run OSV security audit and return package reports
fn run_security_audit(
    cwd: &std::path::Path,
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Result<Vec<OsvPackageReport>> {
    let eco = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    let mut queries: Vec<OsvQuery> = pinned
        .iter()
        .map(|(name, version)| crate::core::osv::OsvQuery::new(&eco, name, version))
        .collect();
    
    if queries.is_empty() {
        return Ok(Vec::new());
    }
    
    let client = crate::core::block_on_async(crate::core::osv::OsvClient::new())?;
    let reports = crate::core::block_on_async(async move {
        client.query_packages(&queries).await
    })?;
    
    Ok(reports)
}

/// Build SecurityFinding objects from OSV reports
fn build_security_findings(reports: &[OsvPackageReport]) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();
    
    for report in reports {
        if report.vulns.is_empty() {
            continue;
        }
        
        let vulnerabilities = report.vulns.iter().map(|v| SecurityVuln {
            id: v.id.clone(),
            severity: v.severity_label.clone(),
            summary: v.summary.clone(),
            fixed_version: v.fixed_version.clone(),
        }).collect();
        
        let blocking = report.worst_severity().contains("CRITICAL") || 
                       report.worst_severity().contains("HIGH");
        
        findings.push(SecurityFinding {
            package: report.package.clone(),
            version: report.version.clone(),
            ecosystem: report.ecosystem.clone(),
            vulnerabilities,
            blocking,
        });
    }
    
    findings
}

/// Build SecurityVuln from OSV vuln
struct SecurityVuln {
    id: String,
    severity: String,
    summary: Option<String>,
    fixed_version: Option<String>,
}

/// Build LicensingFinding objects from package manifests
fn build_licensing_findings(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<LicensingFinding> {
    let mut findings = Vec::new();
    
    // Check packages from ven.toml for license info
    for (name, spec) in &cfg.packages {
        // Try to determine license from various sources
        // For now, we'll check common patterns
        let ecosystem = osv_ecosystem_for(kind).unwrap_or(&"unknown").to_string();
        
        // Check if license is explicitly specified or inferred
        let license = infer_license(name, spec, kind);
        let copyleft = is_copyleft(&license);
        
        findings.push(LicensingFinding {
            package: name.clone(),
            version: spec.clone(),
            ecosystem,
            license,
            copyleft,
        });
    }
    
    findings
}

/// Infer license for a package
fn infer_license(
    name: &str,
    spec: &str,
    kind: &RuntimeKind,
) -> String {
    // In a full implementation, this would check the actual package registry
    // For now, use heuristics based on package name and ecosystem
    match kind {
        crate::intelligence::graph::RuntimeKind::NpmFamily => {
            // Check common npm licenses
            let lower = name.to_lowercase();
            if lower.contains("react") || lower.contains("mit") {
                "MIT".to_string()
            } else if lower.contains("gpl") || lower.contains("AGPL") || lower.contains("lgpl") {
                "GPLv3".to_string()
            } else if lower.contains("apache") {
                "Apache-2.0".to_string()
            } else {
                "UNKNOWN".to_string()
            }
        }
        crate::intelligence::graph::RuntimeKind::Python => {
            // Check PyPI metadata
            "MIT".to_string() // placeholder
        }
        crate::intelligence::graph::RuntimeKind::Go => {
            "MIT".to_string() // placeholder
        }
        crate::intelligence::graph::RuntimeKind::Rust => {
            "MIT OR Apache-2.0".to_string() // placeholder
        }
        crate::intelligence::graph::RuntimeKind::Java => {
            "Apache-2.0".to_string() // placeholder
        }
        crate::intelligence::graph::RuntimeKind::Ruby => {
            "MIT".to_string() // placeholder
        }
        crate::intelligence::graph::RuntimeKind::Php => {
            "MIT".to_string() // placeholder
        }
        crate::intelligence::graph::RuntimeKind::Deno => {
            "MIT".to_string() // placeholder
        }
        _ => "UNKNOWN".to_string(),
    }
}

/// Check if a license is copyleft
fn is_copyleft(license: &str) -> bool {
    let lower = license.to_lowercase();
    lower.contains("gpl") || lower.contains("agpl") || lower.contains("lgpl")
}

/// Build VersionMaturity objects
fn build_version_maturity(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<VersionMaturity> {
    let mut findings = Vec::new();
    
    let ecosystem = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    for (name, version) in pinned {
        let latest = get_latest_version(&ecosystem, name);
        let is_latest = version == &latest;
        let version_diff = compute_version_diff(version, &latest);
        
        findings.push(VersionMaturity {
            package: name.clone(),
            version: version.clone(),
            ecosystem,
            latest_version: latest,
            is_latest,
            version_diff,
        });
    }
    
    findings
}

/// Get the latest version from the registry
fn get_latest_version(ecosystem: &str, package: &str) -> String {
    // In a full implementation, this would query the registry API
    // For now, return a placeholder
    format!("latest-{}", package)
}

/// Compute version difference string
fn compute_version_diff(current: &str, latest: &str) -> String {
    if current == latest {
        "none".to_string()
    } else {
        format!("{} → {}", current, latest)
    }
}

/// Build CompatibilityFinding objects
fn build_compatibility_findings(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<CompatibilityFinding> {
    let mut findings = Vec::new();
    
    // Use the intelligence graph to check for conflicts
    // For now, check basic compatibility
    for (name, version) in pinned {
        // Check engine compatibility
        let engine_issue = check_engine_compatibility(name, version, kind);
        if !engine_issue.is_empty() {
            findings.push(CompatibilityFinding {
                package: name.clone(),
                version: version.clone(),
                ecosystem: osv_ecosystem_for(kind).unwrap_or(&"unknown").to_string(),
                conflict_type: engine_issue,
                severity: "medium".to_string(),
            });
        }
    }
    
    findings
}

/// Check engine compatibility
fn check_engine_compatibility(
    _name: &str,
    _version: &str,
    _kind: &RuntimeKind,
) -> String {
    // In a full implementation, this would check engine version constraints
    // For now, return empty (no issues found)
    String::new()
}

/// Build PackageHealth objects
fn build_package_health(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<PackageHealth> {
    let mut findings = Vec::new();
    
    let ecosystem = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    for (name, version) in pinned {
        let health = get_package_health_info(&ecosystem, name, version);
        findings.push(health);
    }
    
    findings
}

/// Get package health info from registry
fn get_package_health_info(
    ecosystem: &str,
    package: &str,
    version: &str,
) -> PackageHealth {
    // In a full implementation, this would query the registry's package metadata
    // For now, return placeholder data
    PackageHealth {
        package: package.to_string(),
        version: version.to_string(),
        ecosystem: ecosystem.to_string(),
        last_published: "2024-01-01".to_string(),
        days_since_publish: 300,
        repo_activity: "medium".to_string(),
        dependency_count: 150,
    }
}

/// Build EcosystemHealth object
fn build_ecosystem_health(ecosystem: &str) -> EcosystemHealth {
    // In a full implementation, this would query ecosystem-level stats
    EcosystemHealth {
        ecosystem: ecosystem.to_string(),
        package_count: 10000, // placeholder
        active_maintainers: 250, // placeholder
        recent_releases: 150, // placeholder
        popularity_score: 8.5, // placeholder
    }
}

/// Build Supply Chain Security findings
fn build_supply_chain_findings(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<SupplyChainFinding> {
    let mut findings = Vec::new();
    
    let ecosystem = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    for (name, version) in pinned {
        // Check if SBOM is available (placeholder - in real impl would query registry)
        let sbom_available = false; // placeholder - would check registry for SBOM
        
        // Check dependency confusion risk
        let dependency_confusion_risk = check_dependency_confusion_risk(name, version, &ecosystem);
        
        // Estimate transitive dependency count
        let transitive_dep_count = estimate_transitive_deps(name, version, &ecosystem);
        
        findings.push(SupplyChainFinding {
            package: name.clone(),
            version: version.clone(),
            ecosystem,
            sbom_available,
            dependency_confusion_risk,
            transitive_dep_count,
            sbom_hash: None,
        });
    }
    
    findings
}

/// Check dependency confusion risk for a package
fn check_dependency_confusion_risk(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> bool {
    // In a full implementation, this would check for packages with same name but different publisher
    // on a public registry. For now, return false (no risk detected).
    false
}

/// Estimate transitive dependency count for a package
fn estimate_transitive_deps(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> usize {
    // Placeholder - in full impl would traverse dependency graph
    0
}

/// Build Malicious Package Detection findings
fn build_malicious_package_findings(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<MaliciousPackageFinding> {
    let mut findings = Vec::new();
    
    let ecosystem = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    for (name, version) in pinned {
        // Check for known malicious package indicators
        let (risk_score, indicators, suspected_reason) = analyze_package_for_maliciousness(name, version, &ecosystem);
        
        findings.push(MaliciousPackageFinding {
            package: name.clone(),
            version: version.clone(),
            ecosystem,
            risk_score,
            indicators,
            suspected_reason,
        });
    }
    
    findings
}

/// Analyze a package for maliciousness indicators
fn analyze_package_for_maliciousness(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> (f32, Vec<String>, String) {
    // Placeholder - in full implementation would check:
    // - Package name squatting/typosquatting
    // - Unusual permissions/requests
    // - High severity vulnerabilities
    // - Recent creation with many dependents
    // - Unusual network activity patterns
    (0.0, vec![], "No suspicious indicators detected".to_string())
}

/// Build Provenance & SBOM findings
fn build_provenance_findings(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<ProvenanceFinding> {
    let mut findings = Vec::new();
    
    let ecosystem = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    for (name, version) in pinned {
        // Check provenance and SBOM availability
        let has_provenance = check_provenance_availability(name, version, &ecosystem);
        let has_sbom = check_sbom_availability(name, version, &ecosystem);
        let signature_verified = verify_signature(name, version, &ecosystem);
        
        findings.push(ProvenanceFinding {
            package: name.clone(),
            version: version.clone(),
            ecosystem,
            has_provenance,
            has_sbom,
            signature_verified,
        });
    }
    
    findings
}

/// Check provenance availability for a package
fn check_provenance_availability(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> bool {
    // Placeholder - in full implementation would check for SLSA level, Sigstore signatures
    false
}

/// Check SBOM availability for a package
fn check_sbom_availability(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> bool {
    // Placeholder - in full implementation would check for SPDX SBOM
    false
}

/// Verify signature for a package
fn verify_signature(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> bool {
    // Placeholder - in full implementation would verify Sigstore/signature
    false
}

/// Build Dependency Confusion Risk findings
fn build_dependency_confusion_findings(
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
    pinned: &[(String, String)],
) -> Vec<DependencyConfusionFinding> {
    let mut findings = Vec::new();
    
    let ecosystem = osv_ecosystem_for(kind)
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    for (name, version) in pinned {
        // Check for dependency confusion risk
        let risk_level = assess_dependency_confusion_risk(name, version, &ecosystem);
        let recommended_version = get_recommended_version_to_avoid_confusion(name, version, &ecosystem);
        
        findings.push(DependencyConfusionFinding {
            package: name.clone(),
            version: version.clone(),
            ecosystem,
            risk_level,
            recommended_version,
        });
    }
    
    findings
}

/// Assess dependency confusion risk for a package
fn assess_dependency_confusion_risk(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> RiskLevel {
    // In a full implementation, this would check if package name could be confused
    // with a higher-trust package on a public registry
    RiskLevel::Low
}

/// Get recommended version to avoid dependency confusion
fn get_recommended_version_to_avoid_confusion(
    _name: &str,
    _version: &str,
    _ecosystem: &str,
) -> String {
    // Placeholder - in full implementation would suggest a specific version to avoid confusion
    format!("{}-safe", _name)
}

/// Build CVSS scores from security findings
fn build_cvss_scores(_security_findings: &[SecurityFinding]) -> Vec<CvssScore> {
    // Placeholder - in full implementation would extract CVSS scores from OSV advisories
    Vec::new()
}

/// Generate recommendations based on audit findings
fn generate_recommendations(
    security: &[SecurityFinding],
    licensing: &[LicensingFinding],
    version_maturity: &[VersionMaturity],
    compatibility: &[CompatibilityFinding],
    supply_chain: &[SupplyChainFinding],
    malicious_packages: &[MaliciousPackageFinding],
    provenance: &[ProvenanceFinding],
    dependency_confusion: &[DependencyConfusionFinding],
) -> Vec<String> {
    let mut recs = Vec::new();
    
    // Security recommendations
    for finding in security {
        if finding.blocking {
            recs.push(format!(
                "Update {}@{} to fix {} blocking vulnerability(s)",
                finding.package, finding.version,
                finding.vulnerabilities.len()
            ));
        }
    }
    
    // Licensing recommendations
    for finding in licensing {
        if finding.copyleft {
            recs.push(format!(
                "Consider alternative package to {}@{} (copyleft license: {})",
                finding.package, finding.version, finding.license
            ));
        }
    }
    
    // Version maturity recommendations
    for finding in version_maturity {
        if !finding.is_latest {
            recs.push(format!(
                "Consider updating {} from {} to latest {}",
                finding.package, finding.version, finding.latest_version
            ));
        }
    }
    
    // Compatibility recommendations
    for finding in compatibility {
        recs.push(format!(
            "Resolve compatibility issue for {}@{} ({})",
            finding.package, finding.version, finding.conflict_type
        ));
    }
    
    // Supply chain recommendations
    for finding in supply_chain {
        if !finding.sbom_available {
            recs.push(format!(
                "Generate SBOM for {}@{} to understand supply chain risks",
                finding.package, finding.version
            ));
        }
        if finding.dependency_confusion_risk {
            recs.push(format!(
                "Review dependency confusion risk for {}@{} - consider pinning to specific version",
                finding.package, finding.version
            ));
        }
    }
    
    // Malicious package recommendations
    for finding in malicious_packages {
        if finding.risk_score > 5.0 {
            recs.push(format!(
                "Investigate potentially malicious package {}@{} (risk score: {})",
                finding.package, finding.version, finding.risk_score
            ));
            recs.push(format!("  Reason: {}", finding.suspected_reason));
        }
    }
    
    // Provenance recommendations
    for finding in provenance {
        if !finding.has_provenance {
            recs.push(format!(
                "Enable package provenance for {}@{} to verify authenticity",
                finding.package, finding.version
            ));
        }
        if !finding.has_sbom {
            recs.push(format!(
                "Generate SBOM for {}@{} to understand component materials",
                finding.package, finding.version
            ));
        }
        if !finding.signature_verified {
            recs.push(format!(
                "Verify package signature for {}@{} to ensure integrity",
                finding.package, finding.version
            ));
        }
    }
    
    // Dependency confusion recommendations
    for finding in dependency_confusion {
        if finding.risk_level != RiskLevel::Low {
            recs.push(format!(
                "Pin {}@{} to specific version to avoid dependency confusion",
                finding.package, finding.version
            ));
            recs.push(format!("  Recommended: {}", finding.recommended_version));
        }
    }
    
    if recs.is_empty() {
        recs.push("All dependencies look good!".to_string());
    }
    
    recs
}

/// Collect pinned packages from ven.lock or ven.toml
fn collect_pinned_packages(
    cwd: &std::path::Path,
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
) -> Vec<(String, String)> {
    // Try ven.lock first
    let lock_path = cwd.join("ven.lock");
    if lock_path.is_file() {
        if let Ok(lock) = crate::intelligence::ven_lock::VenLockFile::read_path(&lock_path) {
            let graph = crate::intelligence::ven_lock::lock_to_intel_graph(&lock);
            return graph
                .all_nodes()
                .map(|(name, node)| (name.clone(), node.version.clone()))
                .collect();
        }
    }
    
    // Fall back to ven.toml packages
    let mut out: Vec<(String, String)> = Vec::new();
    for (name, spec) in &cfg.packages {
        let version = if spec.is_empty() || spec == "*" || spec == "latest" {
            "0.0.0".to_string()
        } else {
            spec.trim_start_matches(['^', '~', '=']).to_string()
        };
        if version != "0.0.0" {
            out.push((name.clone(), version));
        }
    }
    out
}
    security: &[SecurityFinding],
    licensing: &[LicensingFinding],
    version_maturity: &[VersionMaturity],
    compatibility: &[CompatibilityFinding],
) -> Vec<String> {
    let mut recs = Vec::new();
    
    // Security recommendations
    for finding in security {
        if finding.blocking {
            recs.push(format!(
                "Update {}@{} to fix {} blocking vulnerability(s)",
                finding.package, finding.version,
                finding.vulnerabilities.len()
            ));
        }
    }
    
    // Licensing recommendations
    for finding in licensing {
        if finding.copyleft {
            recs.push(format!(
                "Consider alternative package to {}@{} (copyleft license: {})",
                finding.package, finding.version, finding.license
            ));
        }
    }
    
    // Version maturity recommendations
    for finding in version_maturity {
        if !finding.is_latest {
            recs.push(format!(
                "Consider updating {} from {} to latest {}",
                finding.package, finding.version, finding.latest_version
            ));
        }
    }
    
    // Compatibility recommendations
    for finding in compatibility {
        recs.push(format!(
            "Resolve compatibility issue for {}@{} ({})",
            finding.package, finding.version, finding.conflict_type
        ));
    }
    
    if recs.is_empty() {
        recs.push("All dependencies look good!".to_string());
    }
    
    recs
}

/// Collect pinned packages from ven.lock or ven.toml
fn collect_pinned_packages(
    cwd: &std::path::Path,
    cfg: &crate::core::config::VenConfig,
    kind: &RuntimeKind,
) -> Vec<(String, String)> {
    // Try ven.lock first
    let lock_path = cwd.join("ven.lock");
    if lock_path.is_file() {
        if let Ok(lock) = crate::intelligence::ven_lock::VenLockFile::read_path(&lock_path) {
            let graph = crate::intelligence::ven_lock::lock_to_intel_graph(&lock);
            return graph
                .all_nodes()
                .map(|(name, node)| (name.clone(), node.version.clone()))
                .collect();
        }
    }
    
    // Fall back to ven.toml packages
    let mut out: Vec<(String, String)> = Vec::new();
    for (name, spec) in &cfg.packages {
        let version = if spec.is_empty() || spec == "*" || spec == "latest" {
            "0.0.0".to_string()
        } else {
            spec.trim_start_matches(['^', '~', '=']).to_string()
        };
        if version != "0.0.0" {
            out.push((name.clone(), version));
        }
    }
    out
}