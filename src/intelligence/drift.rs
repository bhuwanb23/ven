//! Drift detection: compare what's locked in `ven.lock` (or declared in
//! `ven.toml`) against what's actually installed on disk.
//!
//! Cross-platform — uses only `std::fs` + `serde_json` for the npm side and
//! shells out to the resolved `python -m pip list --format=json` for the
//! Python side. There is no platform-specific logic here.

use anyhow::Result;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::config::VenConfig;
use crate::intelligence::ven_lock::VenLockFile;

/// Single drift entry for stale installs.
#[derive(Debug, Clone, Serialize)]
pub struct StaleEntry {
    pub package: String,
    pub locked: String,
    pub installed: String,
}

/// Mismatch between `ven.toml [packages]` and what's pinned in `ven.lock`.
#[derive(Debug, Clone, Serialize)]
pub struct ConfigMismatch {
    pub package: String,
    pub ven_toml_spec: String,
    pub lock_pin: String,
}

/// Whole-project drift report. The boolean [`DriftReport::has_drift`] is
/// what `--check` uses to set its exit code.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DriftReport {
    /// In `ven.lock`, missing on disk.
    pub missing: Vec<String>,
    /// In `ven.lock` AND on disk, but at a different version.
    pub stale: Vec<StaleEntry>,
    /// On disk but not in `ven.lock` (transitive deps included — informational).
    pub orphan: Vec<String>,
    /// Roots in `ven.toml [packages]` that don't appear in `ven.lock`.
    pub missing_from_lock: Vec<String>,
    /// Roots whose `ven.toml` constraint can't be satisfied by the lock pin.
    pub config_mismatches: Vec<ConfigMismatch>,
}

impl DriftReport {
    pub fn has_drift(&self) -> bool {
        !self.missing.is_empty()
            || !self.stale.is_empty()
            || !self.missing_from_lock.is_empty()
            || !self.config_mismatches.is_empty()
    }

    /// Orphans alone don't fail `--check` (they're often legitimate
    /// transitive deps of roots npm installs separately).
    pub fn count_actionable(&self) -> usize {
        self.missing.len()
            + self.stale.len()
            + self.missing_from_lock.len()
            + self.config_mismatches.len()
    }
}

/// Compute drift for an npm-family project from its lock + on-disk
/// `node_modules`. Returns an empty report when `node_modules` is absent
/// (caller should warn about that separately — it's a different signal).
pub fn compute_npm_drift(cwd: &Path, lock: &VenLockFile, cfg: &VenConfig) -> Result<DriftReport> {
    let mut report = DriftReport::default();

    // -- root vs lock check ----------------------------------------------
    let lock_root_set: HashSet<&str> = lock.roots.iter().map(|s| s.as_str()).collect();
    for (root, spec) in &cfg.packages {
        if !lock_root_set.contains(root.as_str()) {
            report.missing_from_lock.push(root.clone());
            continue;
        }
        if let Some(pkg) = lock.packages.get(root) {
            if !spec_satisfied_by(spec, &pkg.version) {
                report.config_mismatches.push(ConfigMismatch {
                    package: root.clone(),
                    ven_toml_spec: spec.clone(),
                    lock_pin: pkg.version.clone(),
                });
            }
        }
    }
    report.missing_from_lock.sort();
    report.config_mismatches.sort_by(|a, b| a.package.cmp(&b.package));

    // -- node_modules vs lock --------------------------------------------
    let node_modules = cwd.join("node_modules");
    if !node_modules.is_dir() {
        // Anything in the lock is "missing".
        for name in lock.packages.keys() {
            report.missing.push(name.clone());
        }
        report.missing.sort();
        return Ok(report);
    }

    let installed = read_node_modules_versions(&node_modules)?;

    for (name, pkg) in &lock.packages {
        match installed.get(name) {
            None => report.missing.push(name.clone()),
            Some(on_disk) if on_disk == &pkg.version => {}
            Some(on_disk) => report.stale.push(StaleEntry {
                package: name.clone(),
                locked: pkg.version.clone(),
                installed: on_disk.clone(),
            }),
        }
    }

    let lock_set: HashSet<&str> = lock.packages.keys().map(|s| s.as_str()).collect();
    for name in installed.keys() {
        if !lock_set.contains(name.as_str()) {
            report.orphan.push(name.clone());
        }
    }

    report.missing.sort();
    report.stale.sort_by(|a, b| a.package.cmp(&b.package));
    report.orphan.sort();

    Ok(report)
}

/// Walk one level of `node_modules/`, collecting `pkg → version` pairs from
/// each `package.json`. Handles `@scope/pkg` correctly.
fn read_node_modules_versions(node_modules: &Path) -> Result<HashMap<String, String>> {
    let mut out: HashMap<String, String> = HashMap::new();
    for entry in fs::read_dir(node_modules)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if file_name.starts_with('.') {
            // .bin, .package-lock.json, .cache, ...
            continue;
        }
        if file_name.starts_with('@') {
            // scoped: walk one level deeper
            for sub in fs::read_dir(&path)? {
                let sub = sub?;
                let sub_path = sub.path();
                if !sub_path.is_dir() {
                    continue;
                }
                if let Some(child_name) = sub_path.file_name().and_then(|n| n.to_str()) {
                    let scoped = format!("{}/{}", file_name, child_name);
                    if let Some(v) = read_pkg_version(&sub_path) {
                        out.insert(scoped, v);
                    }
                }
            }
            continue;
        }
        if let Some(v) = read_pkg_version(&path) {
            out.insert(file_name.to_string(), v);
        }
    }
    Ok(out)
}

fn read_pkg_version(pkg_dir: &Path) -> Option<String> {
    let pkg_json = pkg_dir.join("package.json");
    let body = fs::read_to_string(&pkg_json).ok()?;
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    v.get("version")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
}

/// Best-effort semver/`*`/exact match; tolerant of values `ven.toml` typically holds.
fn spec_satisfied_by(spec: &str, pinned: &str) -> bool {
    let spec = spec.trim();
    if spec.is_empty() || spec == "*" || spec == "latest" {
        return true;
    }
    // Exact match first (handles "1.2.3" pins and most non-semver oddities).
    if spec == pinned {
        return true;
    }
    let Ok(pinned_v) = semver::Version::parse(pinned) else {
        // If the pin isn't semver, fall back to literal comparison.
        return spec == pinned;
    };
    if let Ok(req) = semver::VersionReq::parse(spec) {
        return req.matches(&pinned_v);
    }
    if let Ok(major) = spec.parse::<u64>() {
        return pinned_v.major == major;
    }
    false
}

// ── Python drift ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PythonInstalled {
    pub name: String,
    pub version: String,
}

/// Compare `ven.toml [packages]` (and `requirements.txt`) against installed
/// pip packages, using the resolved Python.
pub fn compute_python_drift(
    cwd: &Path,
    cfg: &VenConfig,
    declared_requirements: &[(String, String)],
) -> Result<DriftReport> {
    let mut report = DriftReport::default();

    let installed = pip_list_json(cwd)?;
    let installed_map: HashMap<String, String> = installed
        .into_iter()
        .map(|p| (p.name.to_ascii_lowercase(), p.version))
        .collect();

    // Each ven.toml + requirements entry must be satisfied.
    let mut declared: HashMap<String, String> = HashMap::new();
    for (name, spec) in &cfg.packages {
        declared.insert(name.to_ascii_lowercase(), spec.clone());
    }
    for (name, raw) in declared_requirements {
        declared.entry(name.to_ascii_lowercase()).or_insert_with(|| raw.clone());
    }

    for (name, spec) in &declared {
        match installed_map.get(name) {
            None => report.missing.push(name.clone()),
            Some(installed_v) => {
                if !python_spec_satisfied_by(spec, installed_v) {
                    report.stale.push(StaleEntry {
                        package: name.clone(),
                        locked: spec.clone(),
                        installed: installed_v.clone(),
                    });
                }
            }
        }
    }

    report.missing.sort();
    report.stale.sort_by(|a, b| a.package.cmp(&b.package));
    Ok(report)
}

/// `python -m pip list --format=json` against the resolved interpreter.
fn pip_list_json(cwd: &Path) -> Result<Vec<PythonInstalled>> {
    let python = resolve_python_cmd(cwd);
    let output = Command::new(&python)
        .args(["-m", "pip", "list", "--format=json", "--disable-pip-version-check"])
        .current_dir(cwd)
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let body = String::from_utf8_lossy(&o.stdout);
            let v: Vec<PythonInstalled> = serde_json::from_str(&body).unwrap_or_default();
            Ok(v)
        }
        Ok(_) => Ok(Vec::new()),
        Err(_) => Ok(Vec::new()),
    }
}

fn resolve_python_cmd(cwd: &Path) -> PathBuf {
    if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
        let venv = PathBuf::from(venv);
        #[cfg(target_os = "windows")]
        let candidate = venv.join("Scripts").join("python.exe");
        #[cfg(not(target_os = "windows"))]
        let candidate = venv.join("bin").join("python");
        if candidate.is_file() {
            return candidate;
        }
    }
    // Project-local ./venv / .venv.
    for venv_name in ["venv", ".venv"] {
        let venv_dir = cwd.join(venv_name);
        #[cfg(target_os = "windows")]
        let candidate = venv_dir.join("Scripts").join("python.exe");
        #[cfg(not(target_os = "windows"))]
        let candidate = venv_dir.join("bin").join("python");
        if candidate.is_file() {
            return candidate;
        }
    }
    // ven-managed Python.
    if let Ok(ver) = std::env::var("VEN_PYTHON_VERSION") {
        if let Some(home) = dirs::home_dir() {
            let storage = std::env::var("VEN_STORAGE_PATH")
                .map(PathBuf::from)
                .unwrap_or(home.join(".ven"));
            #[cfg(target_os = "windows")]
            let candidate = storage.join("python").join(&ver).join("python.exe");
            #[cfg(not(target_os = "windows"))]
            let candidate = storage.join("python").join(&ver).join("bin").join("python3");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        PathBuf::from("python")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("python3")
    }
}

/// PEP-440-ish best-effort: handles `==X`, `>=X`, `~=X`, and bare names.
/// Falls back to semver as last resort. Returns `true` when no constraint.
fn python_spec_satisfied_by(spec: &str, installed: &str) -> bool {
    let spec = spec.trim();
    if spec.is_empty() || spec == "*" {
        return true;
    }
    // Strip `name` prefix if user passed full PEP-508 ("requests>=2.32").
    let constraint = match spec.find(|c: char| matches!(c, '<' | '>' | '=' | '~' | '!')) {
        Some(0) => spec.to_string(),
        Some(i) => spec[i..].to_string(),
        None => return true, // bare name → no constraint
    };

    // == exact
    if let Some(rest) = constraint.strip_prefix("==") {
        return rest.trim() == installed;
    }
    // >= / <= / > / < / != / ~=
    let installed_v = match semver::Version::parse(installed) {
        Ok(v) => v,
        Err(_) => return constraint.trim_start_matches("==").trim() == installed,
    };
    if let Some(rest) = constraint.strip_prefix(">=") {
        if let Ok(v) = semver::Version::parse(rest.trim()) {
            return installed_v >= v;
        }
    }
    if let Some(rest) = constraint.strip_prefix("<=") {
        if let Ok(v) = semver::Version::parse(rest.trim()) {
            return installed_v <= v;
        }
    }
    if let Some(rest) = constraint.strip_prefix('>') {
        if let Ok(v) = semver::Version::parse(rest.trim()) {
            return installed_v > v;
        }
    }
    if let Some(rest) = constraint.strip_prefix('<') {
        if let Ok(v) = semver::Version::parse(rest.trim()) {
            return installed_v < v;
        }
    }
    if let Some(rest) = constraint.strip_prefix("!=") {
        if let Ok(v) = semver::Version::parse(rest.trim()) {
            return installed_v != v;
        }
    }
    // Conservative default: assume mismatch.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report_has_no_drift() {
        let r = DriftReport::default();
        assert!(!r.has_drift());
        assert_eq!(r.count_actionable(), 0);
    }

    #[test]
    fn npm_spec_matching() {
        assert!(spec_satisfied_by("*", "1.2.3"));
        assert!(spec_satisfied_by("latest", "9.9.9"));
        assert!(spec_satisfied_by("", "1.2.3"));
        assert!(spec_satisfied_by("^1.2.0", "1.4.0"));
        assert!(spec_satisfied_by("^1.2.0", "1.2.0"));
        assert!(!spec_satisfied_by("^1.2.0", "2.0.0"));
        assert!(spec_satisfied_by("4", "4.18.2"));
        assert!(!spec_satisfied_by("4", "5.0.0"));
        assert!(spec_satisfied_by("1.2.3", "1.2.3"));
    }

    #[test]
    fn python_spec_matching() {
        assert!(python_spec_satisfied_by("requests", "2.32.0"));
        assert!(python_spec_satisfied_by("==2.32.0", "2.32.0"));
        assert!(!python_spec_satisfied_by("==2.32.0", "2.31.0"));
        assert!(python_spec_satisfied_by(">=2.32", "2.32.0"));
        assert!(python_spec_satisfied_by(">=2.32", "2.32.5"));
        assert!(!python_spec_satisfied_by(">=2.32", "2.31.5"));
    }

    #[test]
    fn drift_with_no_node_modules_marks_everything_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = VenConfig::default();
        let lock = VenLockFile {
            lock_format_version: crate::intelligence::ven_lock::LOCK_FORMAT_VERSION,
            ecosystem: "npm".into(),
            runtime_kind: crate::intelligence::graph::RuntimeKind::NpmFamily,
            runtime_version: "20".into(),
            roots: vec!["express".into()],
            packages: HashMap::from([(
                "express".into(),
                crate::intelligence::ven_lock::VenLockPackage {
                    version: "4.18.2".into(),
                    integrity: None,
                    metadata: None,
                },
            )]),
            edges: vec![],
            content_hash: None,
        };
        let report = compute_npm_drift(tmp.path(), &lock, &cfg).unwrap();
        assert_eq!(report.missing, vec!["express"]);
        assert!(report.has_drift());
    }
}
