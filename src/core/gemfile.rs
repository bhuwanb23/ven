//! Line-based reader/writer for `Gemfile`.
//!
//! We don't need a Ruby parser — `gem '<name>'[, '<version>']` lines are easy
//! to detect and edit while preserving everything else (comments, sources,
//! groups, ruby version pins) verbatim.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const FILENAME: &str = "Gemfile";

#[derive(Debug, Clone)]
pub struct Gemfile {
    path: PathBuf,
    lines: Vec<String>,
}

impl Gemfile {
    pub fn path_for(project_dir: &Path) -> PathBuf {
        project_dir.join(FILENAME)
    }

    pub fn load_or_default(project_dir: &Path) -> Result<Self> {
        let path = Self::path_for(project_dir);
        if !path.is_file() {
            return Ok(Self {
                path,
                lines: vec![
                    "source 'https://rubygems.org'".to_string(),
                    String::new(),
                ],
            });
        }
        let body = fs::read_to_string(&path)
            .with_context(|| format!("Read {}", path.display()))?;
        let lines = body.lines().map(|s| s.to_string()).collect();
        Ok(Self { path, lines })
    }

    pub fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Insert or replace a `gem '<name>'[, '<version>']` line.
    /// `version` is the raw spec the caller wants written (e.g. `~> 1.18`,
    /// `>= 6, < 8`). Pass `None` to write a bare `gem '<name>'`.
    pub fn upsert(&mut self, name: &str, version: Option<&str>) {
        let target = format_gem_line(name, version);
        for line in self.lines.iter_mut() {
            if let Some(existing_name) = parse_gem_name(line) {
                if existing_name.eq_ignore_ascii_case(name) {
                    *line = target.clone();
                    return;
                }
            }
        }
        // Append at end (preserve trailing blank line if present)
        if let Some(last) = self.lines.last() {
            if last.is_empty() {
                self.lines.insert(self.lines.len() - 1, target);
                return;
            }
        }
        self.lines.push(target);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.lines.len();
        self.lines.retain(|l| match parse_gem_name(l) {
            Some(existing) => !existing.eq_ignore_ascii_case(name),
            None => true,
        });
        self.lines.len() != before
    }

    pub fn write(&self) -> Result<()> {
        let mut body = self.lines.join("\n");
        if !body.ends_with('\n') {
            body.push('\n');
        }
        fs::write(&self.path, body)
            .with_context(|| format!("Write {}", self.path.display()))?;
        Ok(())
    }

    /// Return all gem entries (name, version_string_or_None) in the file in order.
    pub fn gems(&self) -> Vec<(String, Option<String>)> {
        self.lines
            .iter()
            .filter_map(|l| parse_gem_entry(l))
            .collect()
    }
}

fn format_gem_line(name: &str, version: Option<&str>) -> String {
    match version {
        Some(v) if !v.is_empty() => format!("gem '{}', '{}'", name, v),
        _ => format!("gem '{}'", name),
    }
}

/// Extract the gem name from a single line like `gem 'rails', '~> 7.0'`. Tolerates leading whitespace.
fn parse_gem_name(line: &str) -> Option<&str> {
    parse_gem_entry(line).and_then(|(name, _)| {
        // We need to return a borrowed slice — re-parse to grab one.
        let trimmed = line.trim_start();
        let rest = trimmed.strip_prefix("gem")?;
        let rest = rest.trim_start();
        let (q, after) = strip_quote(rest)?;
        let end = after.find(q)?;
        Some(&after[..end]).filter(|s| s.eq_ignore_ascii_case(&name))
    })
}

fn parse_gem_entry(line: &str) -> Option<(String, Option<String>)> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    let rest = trimmed.strip_prefix("gem")?;
    if rest
        .chars()
        .next()
        .map(|c| !c.is_whitespace())
        .unwrap_or(true)
    {
        return None;
    }
    let rest = rest.trim_start();
    let (q, after) = strip_quote(rest)?;
    let end = after.find(q)?;
    let name = after[..end].to_string();
    let after_name = &after[end + 1..];
    // optional ", '<version>'"
    let v = after_name
        .trim_start()
        .strip_prefix(',')
        .map(str::trim_start)
        .and_then(|s| {
            let (q2, rest2) = strip_quote(s)?;
            let end2 = rest2.find(q2)?;
            Some(rest2[..end2].to_string())
        });
    Some((name, v))
}

fn strip_quote(s: &str) -> Option<(char, &str)> {
    let mut chars = s.chars();
    let q = chars.next()?;
    if q == '"' || q == '\'' {
        Some((q, &s[q.len_utf8()..]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upserts_new_gem_before_trailing_blank() {
        let mut g = Gemfile {
            path: PathBuf::from("/tmp/Gemfile"),
            lines: vec!["source 'https://rubygems.org'".into(), "".into()],
        };
        g.upsert("rails", Some("~> 7.0"));
        assert_eq!(
            g.lines,
            vec!["source 'https://rubygems.org'", "gem 'rails', '~> 7.0'", ""]
        );
    }

    #[test]
    fn upsert_replaces_existing() {
        let mut g = Gemfile {
            path: PathBuf::from("/tmp/Gemfile"),
            lines: vec!["gem 'rails', '6.0'".into()],
        };
        g.upsert("rails", Some("~> 7.0"));
        assert_eq!(g.lines, vec!["gem 'rails', '~> 7.0'"]);
    }

    #[test]
    fn remove_drops_only_target() {
        let mut g = Gemfile {
            path: PathBuf::from("/tmp/Gemfile"),
            lines: vec!["gem 'rails'".into(), "gem 'rspec'".into()],
        };
        assert!(g.remove("rspec"));
        assert_eq!(g.lines, vec!["gem 'rails'"]);
    }

    #[test]
    fn parses_double_quoted_with_version() {
        let (n, v) = parse_gem_entry(r#"  gem "rails", "~> 7.0""#).unwrap();
        assert_eq!(n, "rails");
        assert_eq!(v.as_deref(), Some("~> 7.0"));
    }

    #[test]
    fn ignores_comment() {
        assert!(parse_gem_entry("# gem 'x'").is_none());
    }
}
