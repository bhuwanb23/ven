//! Read/write `requirements.txt` while preserving comments and ordering.
//!
//! Used by `ven add/remove/upgrade <python pkg>` and `ven sync` so the project
//! always has a valid `requirements.txt` next to `ven.toml`.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const FILENAME: &str = "requirements.txt";

#[derive(Debug, Clone)]
pub struct Requirements {
    path: PathBuf,
    lines: Vec<Line>,
}

#[derive(Debug, Clone)]
enum Line {
    Blank,
    Comment(String),
    Pinned { name: String, raw: String },
    Other(String),
}

impl Requirements {
    pub fn path_for(project_dir: &Path) -> PathBuf {
        project_dir.join(FILENAME)
    }

    pub fn load_or_empty(project_dir: &Path) -> Result<Self> {
        let path = Self::path_for(project_dir);
        if !path.is_file() {
            return Ok(Self {
                path,
                lines: Vec::new(),
            });
        }
        let body = fs::read_to_string(&path).with_context(|| format!("Read {}", path.display()))?;
        let mut lines = Vec::new();
        for raw in body.lines() {
            lines.push(parse_line(raw));
        }
        Ok(Self { path, lines })
    }

    /// Insert or replace `name`'s pin line. The full line we write is `raw`
    /// (e.g. `requests>=2.32.0`).
    pub fn upsert(&mut self, name: &str, raw: &str) {
        let lower = name.to_ascii_lowercase();
        for line in self.lines.iter_mut() {
            if let Line::Pinned { name: existing, .. } = line {
                if existing.eq_ignore_ascii_case(&lower) {
                    *line = Line::Pinned {
                        name: lower.clone(),
                        raw: raw.to_string(),
                    };
                    return;
                }
            }
        }
        self.lines.push(Line::Pinned {
            name: lower,
            raw: raw.to_string(),
        });
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        let before = self.lines.len();
        self.lines.retain(|l| match l {
            Line::Pinned { name, .. } => !name.eq_ignore_ascii_case(&lower),
            _ => true,
        });
        self.lines.len() != before
    }

    pub fn write(&self) -> Result<()> {
        let mut out = String::new();
        for (i, l) in self.lines.iter().enumerate() {
            match l {
                Line::Blank => {}
                Line::Comment(s) => out.push_str(s),
                Line::Pinned { raw, .. } => out.push_str(raw),
                Line::Other(s) => out.push_str(s),
            }
            if i + 1 < self.lines.len() {
                out.push('\n');
            }
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }
        fs::write(&self.path, out).with_context(|| format!("Write {}", self.path.display()))?;
        Ok(())
    }

    /// All currently pinned (name, raw) pairs in declared order.
    pub fn pinned(&self) -> Vec<(String, String)> {
        self.lines
            .iter()
            .filter_map(|l| match l {
                Line::Pinned { name, raw } => Some((name.clone(), raw.clone())),
                _ => None,
            })
            .collect()
    }

    pub fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn parse_line(raw: &str) -> Line {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Line::Blank;
    }
    if trimmed.starts_with('#') {
        return Line::Comment(raw.to_string());
    }
    // Treat directives like `-r other.txt`, `-c constraints.txt`, env markers,
    // pip flags as opaque lines we must preserve verbatim.
    if trimmed.starts_with('-') {
        return Line::Other(raw.to_string());
    }
    if let Some(name) = extract_pep508_name(trimmed) {
        return Line::Pinned {
            name: name.to_ascii_lowercase(),
            raw: raw.to_string(),
        };
    }
    Line::Other(raw.to_string())
}

/// Pull the project name out of a PEP-508 line like `requests==2.32.0` or
/// `Flask[async]>=3.0,<4`.
fn extract_pep508_name(line: &str) -> Option<&str> {
    let end = line
        .find(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '[' || c == ']')
        })
        .unwrap_or(line.len());
    let raw = line[..end].trim_end_matches(']');
    let head = raw.split('[').next()?;
    if head.is_empty() {
        return None;
    }
    Some(head)
}

/// Convert a pip install spec like `requests==2.32.0` or `Flask` into the canonical
/// pair `(canonical_name, raw_pin_line)`. If the user typed a bare package name
/// (no version), we emit just the bare name; pip-style requirement files allow
/// unpinned entries.
pub fn requirement_from_spec(spec: &str) -> (String, String) {
    let spec = spec.trim();
    if let Some(name) = extract_pep508_name(spec) {
        return (name.to_ascii_lowercase(), spec.to_string());
    }
    (spec.to_ascii_lowercase(), spec.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_pin() {
        let l = parse_line("requests==2.32.0");
        match l {
            Line::Pinned { name, raw } => {
                assert_eq!(name, "requests");
                assert_eq!(raw, "requests==2.32.0");
            }
            _ => panic!("expected pinned"),
        }
    }

    #[test]
    fn preserves_comments_and_directives() {
        assert!(matches!(parse_line("# comment"), Line::Comment(_)));
        assert!(matches!(parse_line("-r other.txt"), Line::Other(_)));
        assert!(matches!(parse_line(""), Line::Blank));
    }

    #[test]
    fn upsert_replaces_existing() {
        let mut r = Requirements {
            path: PathBuf::from("/tmp/requirements.txt"),
            lines: vec![Line::Pinned {
                name: "requests".into(),
                raw: "requests==2.30.0".into(),
            }],
        };
        r.upsert("requests", "requests==2.32.0");
        assert_eq!(
            r.pinned(),
            vec![("requests".into(), "requests==2.32.0".into())]
        );
    }

    #[test]
    fn remove_drops_only_target() {
        let mut r = Requirements {
            path: PathBuf::from("/tmp/requirements.txt"),
            lines: vec![
                Line::Pinned {
                    name: "requests".into(),
                    raw: "requests".into(),
                },
                Line::Pinned {
                    name: "flask".into(),
                    raw: "flask>=3".into(),
                },
            ],
        };
        assert!(r.remove("requests"));
        let names: Vec<String> = r.pinned().into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["flask".to_string()]);
    }
}
