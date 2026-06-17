//! Read/write the `imports` field of `deno.json` (or `deno.jsonc`).
//!
//! For Deno >= 1.42 we delegate to `deno add` whenever it's on PATH; otherwise
//! we edit the JSON manifest directly while preserving every other key.

use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const FILENAMES: [&str; 2] = ["deno.json", "deno.jsonc"];

#[derive(Debug, Clone)]
pub struct DenoManifest {
    path: PathBuf,
    doc: Value,
}

impl DenoManifest {
    pub fn detect(project_dir: &Path) -> Option<PathBuf> {
        for f in FILENAMES.iter() {
            let p = project_dir.join(f);
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    pub fn load_or_create(project_dir: &Path) -> Result<Self> {
        let path = Self::detect(project_dir).unwrap_or_else(|| project_dir.join("deno.json"));
        let doc = if path.is_file() {
            let body =
                fs::read_to_string(&path).with_context(|| format!("Read {}", path.display()))?;
            // Strip UTF-8 BOM if present (PowerShell / Notepad / VS Code can add one).
            let body = body.strip_prefix('\u{feff}').unwrap_or(&body);
            // deno.jsonc may have // comments — strip a best-effort.
            let stripped = strip_comments_if_jsonc(&path, body);
            let trimmed = stripped.trim();
            if trimmed.is_empty() {
                Value::Object(Map::new())
            } else {
                serde_json::from_str(trimmed)
                    .with_context(|| format!("Parse {}", path.display()))?
            }
        } else {
            Value::Object(Map::new())
        };
        Ok(Self { path, doc })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn imports_mut(&mut self) -> Result<&mut Map<String, Value>> {
        let obj = self
            .doc
            .as_object_mut()
            .ok_or_else(|| anyhow!("deno.json root must be an object"))?;
        if !obj.contains_key("imports") {
            obj.insert("imports".to_string(), Value::Object(Map::new()));
        }
        obj.get_mut("imports")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow!("imports must be a map"))
    }

    pub fn upsert_import(&mut self, key: &str, target: &str) -> Result<()> {
        self.imports_mut()?
            .insert(key.to_string(), Value::String(target.to_string()));
        Ok(())
    }

    pub fn remove_import(&mut self, key: &str) -> Result<bool> {
        let imports = self.imports_mut()?;
        Ok(imports.remove(key).is_some())
    }

    pub fn write(&self) -> Result<()> {
        let body =
            serde_json::to_string_pretty(&self.doc).with_context(|| "serialize deno.json")?;
        fs::write(&self.path, body).with_context(|| format!("Write {}", self.path.display()))?;
        Ok(())
    }
}

fn strip_comments_if_jsonc(path: &Path, body: &str) -> String {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("jsonc"))
        .unwrap_or(false)
    {
        let mut out = String::with_capacity(body.len());
        let mut in_str = false;
        let mut iter = body.chars().peekable();
        while let Some(c) = iter.next() {
            if c == '"' {
                in_str = !in_str;
                out.push(c);
                continue;
            }
            if !in_str && c == '/' {
                if let Some(&'/') = iter.peek() {
                    while let Some(nc) = iter.next() {
                        if nc == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                if let Some(&'*') = iter.peek() {
                    iter.next();
                    while let Some(nc) = iter.next() {
                        if nc == '*' && iter.peek() == Some(&'/') {
                            iter.next();
                            break;
                        }
                    }
                    continue;
                }
            }
            out.push(c);
        }
        out
    } else {
        body.to_string()
    }
}

/// Parse a Deno spec like `npm:react@18` / `jsr:@std/path` / `https://...`.
/// Returns `(import_key, import_target)` for storage in `deno.json`.
pub fn parse_spec(spec: &str) -> Result<(String, String)> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err(anyhow!("empty deno package spec"));
    }
    if let Some(rest) = spec.strip_prefix("npm:") {
        let name = npm_name_from(rest);
        return Ok((name.to_string(), spec.to_string()));
    }
    if let Some(rest) = spec.strip_prefix("jsr:") {
        let name = jsr_name_from(rest);
        return Ok((name.to_string(), spec.to_string()));
    }
    if spec.starts_with("http://") || spec.starts_with("https://") {
        return Ok((spec.to_string(), spec.to_string()));
    }
    // Bare package name → assume jsr if it starts with `@`, else npm.
    if spec.starts_with('@') {
        let key = spec
            .split('@')
            .nth(1)
            .map(|s| format!("@{s}"))
            .unwrap_or_else(|| spec.to_string());
        return Ok((key, format!("jsr:{spec}")));
    }
    Ok((spec.to_string(), format!("npm:{spec}")))
}

fn npm_name_from(rest: &str) -> &str {
    if let Some(stripped) = rest.strip_prefix('@') {
        if let Some(slash) = stripped.find('/') {
            let after = &stripped[slash + 1..];
            let name_end = after.find('@').unwrap_or(after.len());
            let total = 1 + slash + 1 + name_end;
            return &rest[..total];
        }
    }
    rest.split('@').next().unwrap_or(rest)
}

fn jsr_name_from(rest: &str) -> &str {
    if let Some(stripped) = rest.strip_prefix('@') {
        if let Some(slash) = stripped.find('/') {
            let after = &stripped[slash + 1..];
            let name_end = after.find('@').unwrap_or(after.len());
            let total = 1 + slash + 1 + name_end;
            return &rest[..total];
        }
        let name_end = stripped.find('@').unwrap_or(stripped.len());
        return &rest[..1 + name_end];
    }
    rest.split('@').next().unwrap_or(rest)
}

/// Try to invoke `deno add` (Deno >= 1.42). Returns Ok(true) on success,
/// Ok(false) if deno is absent or too old, Err on real failure.
pub fn try_deno_add(specs: &[String]) -> Result<bool> {
    let deno = crate::core::runtime_bin::runtime_tool("deno", "deno");
    let mut cmd = Command::new(&deno);
    cmd.arg("add");
    for s in specs {
        cmd.arg(s);
    }
    match cmd.status() {
        Ok(s) if s.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Ok(false),
    }
}

/// Try to invoke `deno remove`. Same semantics as `try_deno_add`.
pub fn try_deno_remove(names: &[String]) -> Result<bool> {
    let deno = crate::core::runtime_bin::runtime_tool("deno", "deno");
    let mut cmd = Command::new(&deno);
    cmd.arg("remove");
    for n in names {
        cmd.arg(n);
    }
    match cmd.status() {
        Ok(s) if s.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_npm_simple() {
        let (k, v) = parse_spec("npm:react@18").unwrap();
        assert_eq!(k, "react");
        assert_eq!(v, "npm:react@18");
    }

    #[test]
    fn parses_npm_scoped() {
        let (k, v) = parse_spec("npm:@types/node@20").unwrap();
        assert_eq!(k, "@types/node");
        assert_eq!(v, "npm:@types/node@20");
    }

    #[test]
    fn parses_jsr() {
        let (k, v) = parse_spec("jsr:@std/path").unwrap();
        assert_eq!(k, "@std/path");
        assert_eq!(v, "jsr:@std/path");
    }

    #[test]
    fn parses_https_url() {
        let (k, v) = parse_spec("https://deno.land/x/foo@1/mod.ts").unwrap();
        assert_eq!(k, v);
    }

    #[test]
    fn parses_bare_npm_default() {
        let (k, v) = parse_spec("zod").unwrap();
        assert_eq!(k, "zod");
        assert_eq!(v, "npm:zod");
    }
}
