//! `ven audit` — Comprehensive Dependency Risk Report.
//! Provides a multi-dimensional risk assessment for a ven-managed project's dependencies,
//! going beyond just security vulnerabilities to include licensing, version maturity,
//! compatibility, package health, and ecosystem health.

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

// Risk dimension scores and findings
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub vulnerabilities: Vec<VulnerabilityInfo>,
    pub blocking: bool,
}

/// Supply chain security finding
#[derive(Debug, Clone)]
pub struct SupplyChainFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub sbom_available: bool,
    pub dependency_confusion_risk: bool,
    pub transitive_dep_count: usize,
    pub sbom_hash: Option<String>,
}

/// CVSS vulnerability scoring
#[derive(Debug, Clone, PartialEq)]
pub struct CvssScore {
    pub version: String,
    pub severity: String,
    pub cvss3_score: f32,
    pub cvss3_vector: String,
    pub source: String,
}

/// Malicious package detection finding
#[derive(Debug, Clone)]
pub struct MaliciousPackageFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub risk_score: f32,
    pub indicators: Vec<String>,
    pub suspected_reason: String,
}

/// Package provenance finding
#[derive(Debug, Clone)]
pub struct ProvenanceFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub has_provenance: bool,
    pub has_sbom: bool,
    pub signature_verified: bool,
}

/// Dependency confusion risk finding
#[derive(Debug, Clone)]
pub struct DependencyConfusionFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub risk_level: RiskLevel,
    pub recommended_version: String,
}

#[derive(Debug, Clone)]
pub struct VulnerabilityInfo {
    pub id: String,
    pub severity: String,
    pub summary: Option<String>,
    pub fixed_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LicensingFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub license: String,
    pub copyleft: bool,
}

#[derive(Debug, Clone)]
pub struct VersionMaturity {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub latest_version: String,
    pub is_latest: bool,
    pub version_diff: String,
}

#[derive(Debug, Clone)]
pub struct CompatibilityFinding {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub conflict_type: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct PackageHealth {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub last_published: String,
    pub days_since_publish: u64,
    pub repo_activity: String,
    pub dependency_count: usize,
}

#[derive(Debug, Clone)]
pub struct EcosystemHealth {
    pub ecosystem: String,
    pub package_count: usize,
    pub active_maintainers: usize,
    pub recent_releases: usize,
    pub popularity_score: f32,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub ecosystem: String,
    pub security: Vec<SecurityFinding>,
    pub licensing: Vec<LicensingFinding>,
    pub version_maturity: Vec<VersionMaturity>,
    pub compatibility: Vec<CompatibilityFinding>,
    pub package_health: Vec<PackageHealth>,
    pub ecosystem_health: EcosystemHealth,
    /// Supply chain security findings (SBOM availability, dependency confusion risk, provenance)
    pub supply_chain: Vec<SupplyChainFinding>,
    /// CVSS vulnerability scoring
    pub cvss_scores: Vec<CvssScore>,
    /// Malicious package detection findings
    pub malicious_packages: Vec<MaliciousPackageFinding>,
    /// Package provenance findings
    pub provenance: Vec<ProvenanceFinding>,
    /// Dependency confusion risk findings
    pub dependency_confusion: Vec<DependencyConfusionFinding>,
    pub overall_risk_score: f32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }

    pub fn color(&self) -> colored::Color {
        match self {
            RiskLevel::Low => colored::Color::Green,
            RiskLevel::Medium => colored::Color::Yellow,
            RiskLevel::High => colored::Color::Red,
            RiskLevel::Critical => colored::Color::Red,
        }
    }
}

/// Map a vulnerability severity label to a RiskLevel
pub fn severity_to_risk(severity: &str) -> RiskLevel {
    match severity.to_lowercase().as_str() {
        "critical" => RiskLevel::Critical,
        "high" => RiskLevel::High,
        "moderate" => RiskLevel::Medium,
        "low" => RiskLevel::Low,
        _ => RiskLevel::Low,
    }
}

/// Compute an overall risk score from 0.0 to 10.0
pub fn compute_overall_risk_score(
    security_count: usize,
    critical_count: usize,
    licensing_copyleft_count: usize,
    version_outdated_count: usize,
    compatibility_conflicts: usize,
    package_health_concerns: usize,
) -> f32 {
    let mut score = 0.0f32;

    // Security contributes most (max 5 points)
    score += critical_count as f32 * 1.5;
    score += security_count.saturating_sub(critical_count) as f32 * 0.7;
    score = score.min(5.0);

    // Licensing contributes (max 2 points)
    if licensing_copyleft_count > 0 {
        score += 1.0;
        if licensing_copyleft_count > 2 {
            score += 1.0;
        }
    }

    // Version maturity contributes (max 1.5 points)
    score += version_outdated_count as f32 * 0.3;
    score = score.min(6.5);

    // Compatibility contributes (max 1 point)
    score += compatibility_conflicts as f32 * 0.2;
    score = score.min(7.5);

    // Package health contributes (max 2.5 points)
    score += package_health_concerns as f32 * 0.5;
    score = score.min(10.0);

    score
}

/// Determine risk level from score
pub fn risk_level_from_score(score: f32) -> RiskLevel {
    match score {
        s if s >= 8.0 => RiskLevel::Critical,
        s if s >= 6.0 => RiskLevel::High,
        s if s >= 3.0 => RiskLevel::Medium,
        _ => RiskLevel::Low,
    }
}