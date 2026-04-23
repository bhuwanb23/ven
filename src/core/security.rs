use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use colored::Colorize;

/// Security vulnerability advisory from npm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: Option<usize>,
    pub url: Option<String>,
    pub title: String,
    pub severity: String,
    pub vulnerable_versions: String,
    pub module_name: Option<String>,
    pub cves: Vec<String>,
    #[serde(default)]
    pub patched_versions: Option<String>,
}

/// Severity levels for vulnerabilities
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeverityLevel {
    Info,
    Low,
    Moderate,
    High,
    Critical,
}

impl SeverityLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => SeverityLevel::Info,
            "low" => SeverityLevel::Low,
            "moderate" => SeverityLevel::Moderate,
            "high" => SeverityLevel::High,
            "critical" => SeverityLevel::Critical,
            _ => SeverityLevel::Info,
        }
    }

    pub fn display_color(&self) -> colored::ColoredString {
        match self {
            SeverityLevel::Info => "INFO".cyan(),
            SeverityLevel::Low => "LOW".blue(),
            SeverityLevel::Moderate => "MODERATE".yellow(),
            SeverityLevel::High => "HIGH".bright_red(),
            SeverityLevel::Critical => "CRITICAL".red().bold(),
        }
    }
}

/// Security scanner for checking packages against npm advisory database
pub struct SecurityScanner {
    client: Client,
}

impl SecurityScanner {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("ven/0.1.0 (Security Scanner)")
            .build()?;

        Ok(Self { client })
    }

    /// Scan packages for known vulnerabilities using npm Bulk Advisory API
    pub async fn scan_packages(&self, packages: &HashMap<String, String>) -> Result<Vec<Advisory>> {
        if packages.is_empty() {
            return Ok(Vec::new());
        }

        // Build request payload: { "package": ["version1", "version2"] }
        let mut payload: HashMap<String, Vec<String>> = HashMap::new();
        for (name, version) in packages {
            payload.insert(name.clone(), vec![version.clone()]);
        }

        // Query npm Bulk Advisory API
        let url = "https://registry.npmjs.org/-/npm/v1/security/advisories/bulk";
        
        let response = self.client
            .post(url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Security scan failed: HTTP {}",
                response.status()
            ));
        }

        // Parse response: { "package": [advisory1, advisory2] }
        let advisories_map: HashMap<String, Vec<Advisory>> = response.json().await?;

        // Flatten into single list
        let mut all_advisories = Vec::new();
        for (_, advisories) in advisories_map {
            all_advisories.extend(advisories);
        }

        // Sort by severity (critical first)
        all_advisories.sort_by(|a, b| {
            let severity_a = SeverityLevel::from_str(&a.severity);
            let severity_b = SeverityLevel::from_str(&b.severity);
            severity_b.cmp(&severity_a) // Reverse order: critical first
        });

        Ok(all_advisories)
    }

    /// Print security audit results
    pub fn print_audit(&self, advisories: &[Advisory]) {
        if advisories.is_empty() {
            println!("\n  {} No known vulnerabilities found", "✅".green());
            return;
        }

        // Count by severity
        let mut severity_counts = HashMap::new();
        for advisory in advisories {
            let level = SeverityLevel::from_str(&advisory.severity);
            *severity_counts.entry(level).or_insert(0) += 1;
        }

        println!("\n  {} {} vulnerability(ies) found:", "⚠".yellow().bold(), advisories.len());

        for advisory in advisories {
            let severity = SeverityLevel::from_str(&advisory.severity);
            let package = advisory.module_name.as_deref().unwrap_or("unknown");
            
            println!("\n    {} {}: {}", 
                match severity {
                    SeverityLevel::Critical => "🚨".to_string(),
                    SeverityLevel::High => "🔴".to_string(),
                    SeverityLevel::Moderate => "🟡".to_string(),
                    SeverityLevel::Low => "🔵".to_string(),
                    SeverityLevel::Info => "ℹ️".to_string(),
                },
                severity.display_color().bold(),
                package.bold()
            );
            
            println!("      Title: {}", advisory.title);
            println!("      Affected: {}", advisory.vulnerable_versions);
            
            if let Some(ref patched) = advisory.patched_versions {
                println!("      Fixed in: {}", patched);
            }
            
            if let Some(ref url) = advisory.url {
                println!("      Details: {}", url);
            }
        }

        // Summary
        println!("\n  {} Severity Summary:", "📊".cyan());
        for (severity, count) in severity_counts.iter() {
            println!("    {} {}", severity.display_color(), format!("{} vulnerability(ies)", count).dimmed());
        }
    }

    /// Check if any advisory is critical or high severity
    pub fn has_critical_vulnerabilities(&self, advisories: &[Advisory]) -> bool {
        advisories.iter().any(|a| {
            let severity = SeverityLevel::from_str(&a.severity);
            severity == SeverityLevel::Critical || severity == SeverityLevel::High
        })
    }
}
