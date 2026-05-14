//! Ghost dependency scanner — finds packages imported in source but not
//! declared in any project manifest (`ven.toml`, `package.json`,
//! `requirements.txt`, `Cargo.toml`, `go.mod`, `Gemfile`, `pom.xml`,
//! `deno.json`).
//!
//! Cross-platform: pure-Rust regex + the `ignore` crate (gitignore-aware
//! walking). No shell-out, works identically on Windows, macOS, Linux.

use anyhow::Result;
use ignore::WalkBuilder;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::core::config::VenConfig;
use crate::intelligence::graph::RuntimeKind;

/// One detected ghost — a name that appears in source but isn't declared.
#[derive(Debug, Clone, Serialize)]
pub struct Ghost {
    pub name: String,
    pub ecosystem: &'static str,
    /// First file where the ghost was seen (relative to project root).
    pub first_seen_in: String,
    /// Total occurrences across the source tree.
    pub occurrences: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GhostReport {
    pub ecosystem: &'static str,
    pub project_root: String,
    pub ghosts: Vec<Ghost>,
    /// Files visited (after `.gitignore` filtering).
    pub files_scanned: usize,
}

impl GhostReport {
    pub fn has_ghosts(&self) -> bool {
        !self.ghosts.is_empty()
    }
}

/// Scan `cwd` and return ghosts for the project's primary runtime kind.
pub fn scan_project(cwd: &Path, cfg: &VenConfig, kind: RuntimeKind) -> Result<GhostReport> {
    match kind {
        RuntimeKind::NpmFamily => scan_node(cwd, cfg),
        RuntimeKind::Python => scan_python(cwd, cfg),
        RuntimeKind::Go => scan_go(cwd, cfg),
        RuntimeKind::Rust => scan_rust(cwd, cfg),
        RuntimeKind::Java => scan_java(cwd, cfg),
        RuntimeKind::Ruby => scan_ruby(cwd, cfg),
        RuntimeKind::Deno => scan_deno(cwd, cfg),
        RuntimeKind::Stub => Ok(GhostReport {
            ecosystem: "unknown",
            project_root: cwd.to_string_lossy().into_owned(),
            ..Default::default()
        }),
    }
}

// ── helpers ─────────────────────────────────────────────────────────────────

/// Collect declared deps from all known per-ecosystem manifests, plus
/// `ven.toml [packages]`. Stored lower-cased for case-insensitive lookup.
fn collect_declared(cwd: &Path, cfg: &VenConfig, ecosystem: &str) -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();
    for k in cfg.packages.keys() {
        set.insert(k.to_ascii_lowercase());
    }
    match ecosystem {
        "npm" => {
            if let Some(pj) = read_json(cwd.join("package.json")) {
                for key in [
                    "dependencies",
                    "devDependencies",
                    "peerDependencies",
                    "optionalDependencies",
                ] {
                    if let Some(obj) = pj.get(key).and_then(|x| x.as_object()) {
                        for k in obj.keys() {
                            set.insert(k.to_ascii_lowercase());
                        }
                    }
                }
            }
        }
        "pypi" => {
            if let Ok(text) = fs::read_to_string(cwd.join("requirements.txt")) {
                for line in text.lines() {
                    let t = line.trim();
                    if t.is_empty() || t.starts_with('#') || t.starts_with('-') {
                        continue;
                    }
                    if let Some(name) = pep508_name(t) {
                        set.insert(canonical_python_name(name));
                    }
                }
            }
            if let Ok(text) = fs::read_to_string(cwd.join("pyproject.toml")) {
                if let Ok(doc) = text.parse::<toml::Value>() {
                    // PEP-621 `[project] dependencies = ["x>=1", ...]`
                    if let Some(arr) = doc
                        .get("project")
                        .and_then(|p| p.get("dependencies"))
                        .and_then(|d| d.as_array())
                    {
                        for v in arr {
                            if let Some(s) = v.as_str() {
                                if let Some(name) = pep508_name(s) {
                                    set.insert(canonical_python_name(name));
                                }
                            }
                        }
                    }
                }
            }
        }
        "go" => {
            if let Ok(text) = fs::read_to_string(cwd.join("go.mod")) {
                let mut in_block = false;
                for line in text.lines() {
                    let t = line.trim();
                    if t.starts_with("require (") {
                        in_block = true;
                        continue;
                    }
                    if in_block {
                        if t == ")" {
                            in_block = false;
                            continue;
                        }
                        if let Some(name) = t.split_whitespace().next() {
                            set.insert(name.to_ascii_lowercase());
                        }
                    } else if let Some(rest) = t.strip_prefix("require ") {
                        if let Some(name) = rest.split_whitespace().next() {
                            set.insert(name.to_ascii_lowercase());
                        }
                    }
                }
            }
        }
        "crates" => {
            if let Ok(text) = fs::read_to_string(cwd.join("Cargo.toml")) {
                if let Ok(doc) = text.parse::<toml::Value>() {
                    for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
                        if let Some(t) = doc.get(table).and_then(|x| x.as_table()) {
                            for k in t.keys() {
                                set.insert(crate_name_normalize(k));
                            }
                        }
                    }
                }
            }
        }
        "maven" => {
            if let Ok(text) = fs::read_to_string(cwd.join("pom.xml")) {
                // Very lightweight: extract <artifactId>foo</artifactId>.
                let re = artifact_id_regex();
                for cap in re.captures_iter(&text) {
                    if let Some(m) = cap.get(1) {
                        set.insert(m.as_str().to_ascii_lowercase());
                    }
                }
            }
            for gradle in ["build.gradle", "build.gradle.kts"] {
                if let Ok(text) = fs::read_to_string(cwd.join(gradle)) {
                    let re = gradle_dep_regex();
                    for cap in re.captures_iter(&text) {
                        if let Some(m) = cap.get(1) {
                            set.insert(m.as_str().to_ascii_lowercase());
                        }
                    }
                }
            }
        }
        "rubygems" => {
            if let Ok(text) = fs::read_to_string(cwd.join("Gemfile")) {
                let re = gemfile_gem_regex();
                for cap in re.captures_iter(&text) {
                    if let Some(m) = cap.get(1) {
                        set.insert(m.as_str().to_ascii_lowercase());
                    }
                }
            }
        }
        "deno" => {
            if let Some(deno_json) =
                read_json(cwd.join("deno.json")).or_else(|| read_json(cwd.join("deno.jsonc")))
            {
                if let Some(imports) = deno_json.get("imports").and_then(|x| x.as_object()) {
                    for k in imports.keys() {
                        set.insert(k.to_ascii_lowercase());
                    }
                }
            }
        }
        _ => {}
    }
    set
}

fn read_json(path: std::path::PathBuf) -> Option<serde_json::Value> {
    let body = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&body).ok()
}

fn artifact_id_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"<artifactId>\s*([\w\-.]+)\s*</artifactId>").unwrap())
}
fn gradle_dep_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"['"]([a-zA-Z0-9_.-]+):([a-zA-Z0-9_.-]+):[a-zA-Z0-9._\-+]+['"]"#).unwrap()
    })
}
fn gemfile_gem_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"^\s*gem\s+['"]([a-zA-Z0-9_\-]+)['"]"#).unwrap())
}

fn pep508_name(line: &str) -> Option<&str> {
    let end = line
        .find(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '[' || c == ']')
        })
        .unwrap_or(line.len());
    let raw = line[..end].trim_end_matches(']');
    let head = raw.split('[').next()?;
    if head.is_empty() {
        None
    } else {
        Some(head)
    }
}

fn canonical_python_name(s: &str) -> String {
    s.replace('_', "-").to_ascii_lowercase()
}

fn crate_name_normalize(s: &str) -> String {
    s.replace('_', "-").to_ascii_lowercase()
}

/// Build a `WalkBuilder` honoring `.gitignore`, `.ignore`, and our own
/// hard-coded skip list (`node_modules`, `target`, `dist`, `.venv`, …).
fn project_walker(cwd: &Path) -> WalkBuilder {
    let mut wb = WalkBuilder::new(cwd);
    wb.git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .hidden(false)
        .ignore(true)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | ".venv"
                    | "venv"
                    | "__pycache__"
                    | ".tox"
                    | ".git"
                    | ".idea"
                    | ".vscode"
                    | "vendor"
                    | "bower_components"
            )
        });
    wb
}

// ── per-ecosystem scanners ──────────────────────────────────────────────────

fn scan_node(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "npm");
    let stdlib = node_stdlib();
    let exts: HashSet<&str> = ["js", "mjs", "cjs", "jsx", "ts", "tsx"]
        .into_iter()
        .collect();
    let import_re = node_import_regex();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;

    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| exts.contains(e))
        {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };
        for cap in import_re.captures_iter(&body) {
            let spec = cap.get(1).or_else(|| cap.get(2)).or_else(|| cap.get(3));
            let Some(spec) = spec else { continue };
            let name = node_extract_name(spec.as_str());
            if name.is_empty() {
                continue;
            }
            // skip node:fs, node:path, etc.
            if let Some(rest) = name.strip_prefix("node:") {
                if stdlib.contains(rest) {
                    continue;
                }
            }
            if stdlib.contains(name.as_str()) {
                continue;
            }
            if declared.contains(&name.to_ascii_lowercase()) {
                continue;
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences.entry(name).or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "npm",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "npm",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}

fn node_import_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        // Captures both `import ... from 'spec'` (and bare-side-effect
        // `import 'spec'`) and `require('spec')` and `await import('spec')`.
        // Single capture group covers all four shapes.
        Regex::new(
            r#"(?m)(?:require\(\s*['"]([^'"]+)['"]\s*\)|import\s*\(?\s*['"]([^'"]+)['"]|from\s+['"]([^'"]+)['"])"#,
        )
        .unwrap()
    })
}

/// We use a multi-group regex; pull the non-empty match out.
fn node_extract_name(_first_capture: &str) -> String {
    // The regex above has 3 alternation groups; the iterator returns the
    // first non-None one in `cap.get(1..=3)`. Caller passes the matched
    // string from get(1); we still need to handle scope/subpath here.
    let raw = _first_capture.trim();
    if raw.is_empty() || raw.starts_with('.') || raw.starts_with('/') {
        return String::new();
    }
    if raw.starts_with('@') {
        let mut parts = raw.splitn(3, '/');
        match (parts.next(), parts.next()) {
            (Some(scope), Some(pkg)) if !pkg.is_empty() => format!("{}/{}", scope, pkg),
            _ => raw.to_string(),
        }
    } else {
        raw.split('/').next().unwrap_or(raw).to_string()
    }
}

fn node_stdlib() -> HashSet<&'static str> {
    [
        "assert",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "diagnostics_channel",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "http2",
        "https",
        "inspector",
        "module",
        "net",
        "os",
        "path",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "string_decoder",
        "sys",
        "timers",
        "tls",
        "trace_events",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "wasi",
        "worker_threads",
        "zlib",
    ]
    .into_iter()
    .collect()
}

fn scan_python(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "pypi");
    let stdlib = python_stdlib();
    let import_re = python_import_regex();
    let rename = python_rename_table();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;

    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("py") {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };
        for cap in import_re.captures_iter(&body) {
            let Some(m) = cap.get(1) else { continue };
            let top = m
                .as_str()
                .split('.')
                .next()
                .unwrap_or(m.as_str())
                .to_string();
            if stdlib.contains(top.as_str()) {
                continue;
            }
            let pip_name = rename
                .get(top.as_str())
                .copied()
                .unwrap_or(top.as_str())
                .to_ascii_lowercase();
            let canonical = canonical_python_name(&pip_name);
            if declared.contains(&canonical) {
                continue;
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences
                .entry(canonical)
                .or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "pypi",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "pypi",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}

fn python_import_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^\s*(?:from|import)\s+([\w.]+)").unwrap())
}

fn python_rename_table() -> HashMap<&'static str, &'static str> {
    [
        ("cv2", "opencv-python"),
        ("PIL", "Pillow"),
        ("yaml", "PyYAML"),
        ("sklearn", "scikit-learn"),
        ("bs4", "beautifulsoup4"),
        ("dateutil", "python-dateutil"),
        ("dotenv", "python-dotenv"),
        ("Crypto", "pycryptodome"),
        ("OpenGL", "PyOpenGL"),
    ]
    .into_iter()
    .collect()
}

fn python_stdlib() -> HashSet<&'static str> {
    // Trimmed but covers the bulk. Add as needed.
    [
        "abc",
        "argparse",
        "array",
        "ast",
        "asyncio",
        "atexit",
        "base64",
        "binascii",
        "bisect",
        "builtins",
        "calendar",
        "cgi",
        "cmath",
        "cmd",
        "code",
        "codecs",
        "collections",
        "colorsys",
        "concurrent",
        "configparser",
        "contextlib",
        "contextvars",
        "copy",
        "copyreg",
        "csv",
        "ctypes",
        "curses",
        "dataclasses",
        "datetime",
        "decimal",
        "difflib",
        "dis",
        "doctest",
        "email",
        "encodings",
        "enum",
        "errno",
        "fcntl",
        "filecmp",
        "fileinput",
        "fnmatch",
        "fractions",
        "ftplib",
        "functools",
        "gc",
        "getopt",
        "getpass",
        "gettext",
        "glob",
        "grp",
        "gzip",
        "hashlib",
        "heapq",
        "hmac",
        "html",
        "http",
        "imaplib",
        "imghdr",
        "importlib",
        "inspect",
        "io",
        "ipaddress",
        "itertools",
        "json",
        "keyword",
        "linecache",
        "locale",
        "logging",
        "lzma",
        "mailbox",
        "math",
        "mimetypes",
        "mmap",
        "multiprocessing",
        "netrc",
        "nntplib",
        "numbers",
        "operator",
        "optparse",
        "os",
        "pathlib",
        "pdb",
        "pickle",
        "platform",
        "plistlib",
        "poplib",
        "posix",
        "pprint",
        "profile",
        "pstats",
        "pwd",
        "py_compile",
        "pydoc",
        "queue",
        "quopri",
        "random",
        "re",
        "readline",
        "reprlib",
        "resource",
        "rlcompleter",
        "runpy",
        "sched",
        "secrets",
        "select",
        "selectors",
        "shelve",
        "shlex",
        "shutil",
        "signal",
        "site",
        "smtplib",
        "sndhdr",
        "socket",
        "socketserver",
        "sqlite3",
        "ssl",
        "stat",
        "statistics",
        "string",
        "stringprep",
        "struct",
        "subprocess",
        "sunau",
        "symtable",
        "sys",
        "sysconfig",
        "syslog",
        "tabnanny",
        "tarfile",
        "telnetlib",
        "tempfile",
        "termios",
        "test",
        "textwrap",
        "threading",
        "time",
        "timeit",
        "tkinter",
        "token",
        "tokenize",
        "trace",
        "traceback",
        "tracemalloc",
        "tty",
        "turtle",
        "types",
        "typing",
        "unicodedata",
        "unittest",
        "urllib",
        "uu",
        "uuid",
        "venv",
        "warnings",
        "wave",
        "weakref",
        "webbrowser",
        "winreg",
        "winsound",
        "wsgiref",
        "xml",
        "xmlrpc",
        "zipapp",
        "zipfile",
        "zipimport",
        "zlib",
        "zoneinfo",
        "__future__",
        "__main__",
    ]
    .into_iter()
    .collect()
}

fn scan_go(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "go");
    let import_re = go_import_regex();
    let block_re = go_import_block_regex();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;

    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("go") {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };

        let mut paths: Vec<String> = Vec::new();
        for cap in import_re.captures_iter(&body) {
            if let Some(m) = cap.get(1) {
                paths.push(m.as_str().to_string());
            }
        }
        for block_cap in block_re.captures_iter(&body) {
            if let Some(block) = block_cap.get(1) {
                for line in block.as_str().lines() {
                    let t = line.trim();
                    if t.is_empty() || t.starts_with("//") {
                        continue;
                    }
                    // strip optional alias: `_ "fmt"` or `f "fmt"`
                    let after_alias = t.find('"').and_then(|start| {
                        let rest = &t[start + 1..];
                        rest.find('"').map(|end| rest[..end].to_string())
                    });
                    if let Some(p) = after_alias {
                        paths.push(p);
                    }
                }
            }
        }

        for spec in paths {
            // Pure-stdlib paths have no dot in their first segment.
            let first_seg = spec.split('/').next().unwrap_or("");
            if !first_seg.contains('.') {
                continue;
            }
            // Module path is typically host/owner/repo[/...]; ven.toml /
            // go.mod records the first three segments (or the whole path if
            // shorter).
            let module_path = go_module_root(&spec);
            if declared.contains(&module_path.to_ascii_lowercase()) {
                continue;
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences
                .entry(module_path)
                .or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "go",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "go",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}

fn go_import_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"(?m)^\s*import\s+(?:[\w_]+\s+)?"([^"]+)""#).unwrap())
}
fn go_import_block_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?ms)^\s*import\s*\(\s*(.*?)\s*\)").unwrap())
}

fn go_module_root(spec: &str) -> String {
    let parts: Vec<&str> = spec.split('/').collect();
    let take = parts.len().min(3);
    parts[..take].join("/")
}

fn scan_rust(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "crates");
    let stdlib = rust_stdlib();
    let use_re = rust_use_regex();
    let extern_re = rust_extern_regex();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;

    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };

        let mut names: BTreeSet<String> = BTreeSet::new();
        for cap in use_re.captures_iter(&body) {
            if let Some(m) = cap.get(1) {
                names.insert(m.as_str().to_string());
            }
        }
        for cap in extern_re.captures_iter(&body) {
            if let Some(m) = cap.get(1) {
                names.insert(m.as_str().to_string());
            }
        }

        for ident in names {
            if stdlib.contains(ident.as_str()) {
                continue;
            }
            let crate_name = crate_name_normalize(&ident);
            if declared.contains(&crate_name) {
                continue;
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences
                .entry(crate_name)
                .or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "crates.io",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "crates.io",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}

fn rust_use_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^\s*use\s+([a-zA-Z_][\w]*)::").unwrap())
}
fn rust_extern_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^\s*extern\s+crate\s+([a-zA-Z_][\w]*)").unwrap())
}
fn rust_stdlib() -> HashSet<&'static str> {
    ["std", "core", "alloc", "self", "super", "crate", "test"]
        .into_iter()
        .collect()
}

fn scan_java(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "maven");
    let import_re = java_import_regex();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;

    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("java") {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };
        for cap in import_re.captures_iter(&body) {
            let Some(full) = cap.get(1) else { continue };
            let parts: Vec<&str> = full.as_str().split('.').collect();
            if parts.len() < 2 {
                continue;
            }
            // Skip JDK packages: java.*, javax.*, jdk.*, sun.*, com.sun.*.
            if matches!(parts[0], "java" | "javax" | "jdk" | "sun") {
                continue;
            }
            if parts.len() >= 2 && parts[0] == "com" && parts[1] == "sun" {
                continue;
            }
            // Best-effort artifact key: top-level reverse-DNS prefix
            // (`com.fasterxml`, `org.apache.commons`).
            let probe = if parts.len() >= 2 {
                format!("{}.{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            };
            let probe_lower = probe.to_ascii_lowercase();
            if declared.iter().any(|d| d.contains(&probe_lower)) {
                continue;
            }
            // Show last meaningful segment (artifactId-ish) to match Maven pom
            // listings: `org.apache.commons.lang3.StringUtils` → `lang3`.
            let candidate = parts
                .get(parts.len().saturating_sub(2))
                .copied()
                .unwrap_or(&parts[parts.len() - 1])
                .to_ascii_lowercase();
            if declared.contains(&candidate) {
                continue;
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences
                .entry(candidate)
                .or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "maven",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "maven",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}
fn java_import_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^\s*import\s+(?:static\s+)?([\w.]+);").unwrap())
}

fn scan_ruby(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "rubygems");
    let stdlib = ruby_stdlib();
    let req_re = ruby_require_regex();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;

    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rb") {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };
        for cap in req_re.captures_iter(&body) {
            let Some(m) = cap.get(1) else { continue };
            let raw = m.as_str();
            // Take the first path segment as gem name.
            let name = raw.split('/').next().unwrap_or(raw).to_ascii_lowercase();
            if stdlib.contains(name.as_str()) {
                continue;
            }
            if declared.contains(&name) {
                continue;
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences.entry(name).or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "rubygems",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "rubygems",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}
fn ruby_require_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"(?m)^\s*require\s+['"]([^./][^'"]+)['"]"#).unwrap())
}
fn ruby_stdlib() -> HashSet<&'static str> {
    [
        "set",
        "json",
        "yaml",
        "csv",
        "uri",
        "net",
        "fileutils",
        "pathname",
        "tempfile",
        "time",
        "date",
        "digest",
        "openssl",
        "base64",
        "stringio",
        "logger",
        "optparse",
        "ostruct",
        "securerandom",
        "socket",
        "thread",
        "weakref",
    ]
    .into_iter()
    .collect()
}

fn scan_deno(cwd: &Path, cfg: &VenConfig) -> Result<GhostReport> {
    let declared = collect_declared(cwd, cfg, "deno");
    let import_re = deno_import_regex();

    let mut occurrences: HashMap<String, (usize, String)> = HashMap::new();
    let mut files_scanned = 0usize;
    for entry in project_walker(cwd).build() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(ext, "ts" | "tsx" | "js" | "mjs") {
            continue;
        }
        files_scanned += 1;
        let Ok(body) = fs::read_to_string(path) else {
            continue;
        };
        for cap in import_re.captures_iter(&body) {
            let spec = cap.get(1).or_else(|| cap.get(2));
            let Some(spec) = spec else { continue };
            let raw = spec.as_str();
            // skip relative
            if raw.starts_with('.') || raw.starts_with('/') {
                continue;
            }
            // Group npm: + jsr: + plain URL imports as ghosts.
            // import-map keys land in `imports` and are honored via `declared`.
            let key_for_lookup = raw.to_ascii_lowercase();
            if declared.contains(&key_for_lookup) {
                continue;
            }
            // For npm:foo@1 form, try the bare `npm:foo` key too.
            if let Some(rest) = raw.strip_prefix("npm:") {
                let bare = rest.split('@').next().unwrap_or(rest);
                if declared.contains(&format!("npm:{}", bare).to_ascii_lowercase()) {
                    continue;
                }
            }
            let rel = path
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let (count, first) = occurrences
                .entry(raw.to_string())
                .or_insert_with(|| (0, rel.clone()));
            *count += 1;
            if first.is_empty() {
                *first = rel;
            }
        }
    }

    let mut ghosts: Vec<Ghost> = occurrences
        .into_iter()
        .map(|(name, (count, first))| Ghost {
            name,
            ecosystem: "deno",
            first_seen_in: first,
            occurrences: count,
        })
        .collect();
    ghosts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(GhostReport {
        ecosystem: "deno",
        project_root: cwd.to_string_lossy().into_owned(),
        ghosts,
        files_scanned,
    })
}
fn deno_import_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"(?m)(?:from\s+['"]([^'"]+)['"]|import\s*\(?\s*['"]([^'"]+)['"])"#).unwrap()
    })
}

// ── Updated node ghost regex implementation: deal with multi-group captures ──

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, text: &str) {
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(p, text).unwrap();
    }

    #[test]
    fn node_extract_handles_scopes_and_subpaths() {
        assert_eq!(node_extract_name("express"), "express");
        assert_eq!(node_extract_name("lodash/fp"), "lodash");
        assert_eq!(node_extract_name("@scope/pkg"), "@scope/pkg");
        assert_eq!(node_extract_name("@scope/pkg/sub"), "@scope/pkg");
        assert_eq!(node_extract_name("./relative"), "");
        assert_eq!(node_extract_name("/abs"), "");
    }

    #[test]
    fn pep508_name_extraction() {
        assert_eq!(pep508_name("requests==2.32.0"), Some("requests"));
        assert_eq!(pep508_name("Flask[async]>=3.0,<4"), Some("Flask"));
        assert_eq!(pep508_name(""), None);
    }

    #[test]
    fn go_module_root_takes_first_three_segments() {
        assert_eq!(
            go_module_root("github.com/foo/bar/sub"),
            "github.com/foo/bar"
        );
        assert_eq!(go_module_root("example.com/foo"), "example.com/foo");
    }

    #[test]
    fn pure_stdlib_node_skipped() {
        let stdlib = node_stdlib();
        assert!(stdlib.contains("fs"));
        assert!(stdlib.contains("crypto"));
        assert!(!stdlib.contains("express"));
    }

    #[test]
    fn scan_node_finds_ghosts() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        // declared: only `express`
        write(
            &cwd.join("package.json"),
            r#"{"dependencies":{"express":"4.0.0"}}"#,
        );
        // src/ requires express (declared) + axios (ghost) + node:fs (stdlib)
        write(
            &cwd.join("src/app.js"),
            r#"
const express = require("express");
const axios = require("axios");
const fs = require("node:fs");
import lodash from 'lodash';
"#,
        );
        let cfg = VenConfig::default();
        let report = scan_node(cwd, &cfg).unwrap();
        let names: Vec<&str> = report.ghosts.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"axios"), "expected axios in {:?}", names);
        assert!(names.contains(&"lodash"), "expected lodash in {:?}", names);
        assert!(!names.contains(&"express"));
        assert!(!names.contains(&"fs"));
    }

    #[test]
    fn scan_python_finds_ghosts() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        write(&cwd.join("requirements.txt"), "requests==2.32.0\n");
        write(
            &cwd.join("app.py"),
            "import requests\nimport flask\nfrom os import path\n",
        );
        let cfg = VenConfig::default();
        let report = scan_python(cwd, &cfg).unwrap();
        let names: Vec<&str> = report.ghosts.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"flask"), "expected flask in {:?}", names);
        assert!(!names.contains(&"requests"));
        assert!(!names.contains(&"os"));
    }

    #[test]
    fn scan_rust_finds_ghosts() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        write(
            &cwd.join("Cargo.toml"),
            r#"[package]
name="x"
version="0.1.0"
[dependencies]
serde = "1"
"#,
        );
        write(
            &cwd.join("src/main.rs"),
            "use serde::Serialize;\nuse anyhow::Result;\nuse std::collections::HashMap;\n",
        );
        let cfg = VenConfig::default();
        let report = scan_rust(cwd, &cfg).unwrap();
        let names: Vec<&str> = report.ghosts.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"anyhow"));
        assert!(!names.contains(&"serde"));
        assert!(!names.contains(&"std"));
    }
}
