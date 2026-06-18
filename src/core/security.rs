use anyhow::{anyhow, Result};
use colored::Colorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

        let response = self.client.post(url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Security scan failed: HTTP {}", response.status()));
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
            println!("\n  {} No known vulnerabilities found", "[OK]".green());
            return;
        }

        // Count by severity
        let mut severity_counts = HashMap::new();
        for advisory in advisories {
            let level = SeverityLevel::from_str(&advisory.severity);
            *severity_counts.entry(level).or_insert(0) += 1;
        }

        println!(
            "\n  {} {} vulnerability(ies) found:",
            "[WARN]".yellow().bold(),
            advisories.len()
        );

        for advisory in advisories {
            let severity = SeverityLevel::from_str(&advisory.severity);
            let package = advisory.module_name.as_deref().unwrap_or("unknown");

            println!(
                "\n    {} {}: {}",
                match severity {
                    SeverityLevel::Critical => "[CRIT]".to_string(),
                    SeverityLevel::High => "[HIGH]".to_string(),
                    SeverityLevel::Moderate => "[MOD]".to_string(),
                    SeverityLevel::Low => "[LOW]".to_string(),
                    SeverityLevel::Info => "[INFO]".to_string(),
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
        println!("\n  {} Severity Summary:", "[SUMMARY]".cyan());
        for (severity, count) in severity_counts.iter() {
            println!(
                "    {} {}",
                severity.display_color(),
                format!("{} vulnerability(ies)", count).dimmed()
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_from_str_critical() {
        assert_eq!(SeverityLevel::from_str("critical"), SeverityLevel::Critical);
    }

    #[test]
    fn severity_from_str_high() {
        assert_eq!(SeverityLevel::from_str("high"), SeverityLevel::High);
    }

    #[test]
    fn severity_from_str_moderate() {
        assert_eq!(SeverityLevel::from_str("moderate"), SeverityLevel::Moderate);
    }

    #[test]
    fn severity_from_str_low() {
        assert_eq!(SeverityLevel::from_str("low"), SeverityLevel::Low);
    }

    #[test]
    fn severity_from_str_info() {
        assert_eq!(SeverityLevel::from_str("info"), SeverityLevel::Info);
    }

    #[test]
    fn severity_from_str_unknown_defaults_to_info() {
        assert_eq!(SeverityLevel::from_str("something_weird"), SeverityLevel::Info);
        assert_eq!(SeverityLevel::from_str(""), SeverityLevel::Info);
    }

    #[test]
    fn severity_from_str_case_insensitive() {
        assert_eq!(SeverityLevel::from_str("CRITICAL"), SeverityLevel::Critical);
        assert_eq!(SeverityLevel::from_str("High"), SeverityLevel::High);
        assert_eq!(SeverityLevel::from_str("MODERATE"), SeverityLevel::Moderate);
    }

    #[test]
    fn severity_ordering() {
        assert!(SeverityLevel::Critical > SeverityLevel::High);
        assert!(SeverityLevel::High > SeverityLevel::Moderate);
        assert!(SeverityLevel::Moderate > SeverityLevel::Low);
        assert!(SeverityLevel::Low > SeverityLevel::Info);
    }

    #[test]
    fn advisory_deserialize() {
        let json = r#"{
            "id": 123,
            "url": "https://example.com/advisory",
            "title": "Test vulnerability",
            "severity": "high",
            "vulnerable_versions": "< 2.0.0",
            "module_name": "test-pkg",
            "cves": ["CVE-2024-0001"],
            "patched_versions": ">= 2.0.0"
        }"#;

        let advisory: Advisory = serde_json::from_str(json).unwrap();
        assert_eq!(advisory.id, Some(123));
        assert_eq!(advisory.title, "Test vulnerability");
        assert_eq!(advisory.severity, "high");
        assert_eq!(advisory.module_name.as_deref(), Some("test-pkg"));
        assert_eq!(advisory.cves, vec!["CVE-2024-0001"]);
        assert_eq!(advisory.patched_versions.as_deref(), Some(">= 2.0.0"));
    }

    #[test]
    fn advisory_deserialize_minimal_fields() {
        let json = r#"{
            "title": "Minimal advisory",
            "severity": "low",
            "vulnerable_versions": "*",
            "cves": []
        }"#;

        let advisory: Advisory = serde_json::from_str(json).unwrap();
        assert_eq!(advisory.id, None);
        assert_eq!(advisory.url, None);
        assert_eq!(advisory.module_name, None);
        assert_eq!(advisory.patched_versions, None);
    }

    #[test]
    fn advisory_roundtrip() {
        let advisory = Advisory {
            id: Some(42),
            url: None,
            title: "Roundtrip test".into(),
            severity: "moderate".into(),
            vulnerable_versions: "< 1.0".into(),
            module_name: Some("foo".into()),
            cves: vec![],
            patched_versions: Some(">= 1.0".into()),
        };

        let json = serde_json::to_string(&advisory).unwrap();
        let deserialized: Advisory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "Roundtrip test");
        assert_eq!(deserialized.severity, "moderate");
    }

    fn make_advisory(severity: &str, name: &str) -> Advisory {
        Advisory {
            id: None,
            url: None,
            title: format!("Advisory for {}", name),
            severity: severity.into(),
            vulnerable_versions: "*".into(),
            module_name: Some(name.into()),
            cves: vec![],
            patched_versions: None,
        }
    }

    #[test]
    fn has_critical_vulnerabilities_with_critical() {
        let scanner = SecurityScanner::new().unwrap();
        let advisories = vec![make_advisory("critical", "pkg")];
        assert!(scanner.has_critical_vulnerabilities(&advisories));
    }

    #[test]
    fn has_critical_vulnerabilities_with_high() {
        let scanner = SecurityScanner::new().unwrap();
        let advisories = vec![make_advisory("high", "pkg")];
        assert!(scanner.has_critical_vulnerabilities(&advisories));
    }

    #[test]
    fn has_critical_vulnerabilities_with_low_only() {
        let scanner = SecurityScanner::new().unwrap();
        let advisories = vec![
            make_advisory("low", "pkg"),
            make_advisory("info", "pkg2"),
        ];
        assert!(!scanner.has_critical_vulnerabilities(&advisories));
    }

    #[test]
    fn has_critical_vulnerabilities_empty() {
        let scanner = SecurityScanner::new().unwrap();
        assert!(!scanner.has_critical_vulnerabilities(&[]));
    }

    #[test]
    fn has_critical_vulnerabilities_mixed() {
        let scanner = SecurityScanner::new().unwrap();
        let advisories = vec![
            make_advisory("low", "safe-pkg"),
            make_advisory("moderate", "ok-pkg"),
            make_advisory("high", "bad-pkg"),
        ];
        assert!(scanner.has_critical_vulnerabilities(&advisories));
    }
}
