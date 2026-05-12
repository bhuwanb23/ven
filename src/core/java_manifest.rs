//! Detect Maven (`pom.xml`) and Gradle (`build.gradle` / `build.gradle.kts`)
//! projects, and add/remove/upgrade dependencies in their manifest files.
//!
//! `pom.xml` editing is line-based and conservative: insert `<dependency>` tags
//! into an existing `<dependencies>` block, never reformat. Gradle is purely
//! line-based — append `implementation '<group>:<artifact>:<ver>'` (or the
//! Kotlin form for `.kts`) inside the `dependencies { ... }` block.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTool {
    Maven,
    GradleGroovy,
    GradleKotlin,
}

#[derive(Debug, Clone)]
pub struct JavaProject {
    pub tool: BuildTool,
    pub manifest: PathBuf,
}

#[derive(Debug, Clone)]
pub struct JavaCoord {
    pub group: String,
    pub artifact: String,
    pub version: Option<String>,
}

impl JavaCoord {
    /// Parse `group:artifact[@version]` or `group:artifact:version`.
    pub fn parse(spec: &str) -> Result<Self> {
        let (left, version) = if let Some((l, v)) = spec.rsplit_once('@') {
            (l, Some(v.to_string()))
        } else {
            // Allow Maven-style `group:artifact:version`.
            let parts: Vec<&str> = spec.split(':').collect();
            if parts.len() == 3 {
                return Ok(JavaCoord {
                    group: parts[0].to_string(),
                    artifact: parts[1].to_string(),
                    version: Some(parts[2].to_string()),
                });
            }
            (spec, None)
        };
        let (group, artifact) = left
            .split_once(':')
            .ok_or_else(|| anyhow!("Java coordinate must be `group:artifact[@version]`: {}", spec))?;
        if group.trim().is_empty() || artifact.trim().is_empty() {
            return Err(anyhow!(
                "Java coordinate must be `group:artifact[@version]`: {}",
                spec
            ));
        }
        Ok(JavaCoord {
            group: group.to_string(),
            artifact: artifact.to_string(),
            version,
        })
    }

    pub fn ven_toml_key(&self) -> String {
        format!("{}:{}", self.group, self.artifact)
    }
}

pub fn detect(project_dir: &Path) -> Option<JavaProject> {
    let pom = project_dir.join("pom.xml");
    if pom.is_file() {
        return Some(JavaProject {
            tool: BuildTool::Maven,
            manifest: pom,
        });
    }
    let kts = project_dir.join("build.gradle.kts");
    if kts.is_file() {
        return Some(JavaProject {
            tool: BuildTool::GradleKotlin,
            manifest: kts,
        });
    }
    let groovy = project_dir.join("build.gradle");
    if groovy.is_file() {
        return Some(JavaProject {
            tool: BuildTool::GradleGroovy,
            manifest: groovy,
        });
    }
    None
}

pub fn add(project: &JavaProject, coord: &JavaCoord) -> Result<()> {
    match project.tool {
        BuildTool::Maven => maven_add(&project.manifest, coord),
        BuildTool::GradleGroovy => gradle_add(&project.manifest, coord, false),
        BuildTool::GradleKotlin => gradle_add(&project.manifest, coord, true),
    }
}

pub fn remove(project: &JavaProject, coord: &JavaCoord) -> Result<bool> {
    match project.tool {
        BuildTool::Maven => maven_remove(&project.manifest, coord),
        BuildTool::GradleGroovy => gradle_remove(&project.manifest, coord, false),
        BuildTool::GradleKotlin => gradle_remove(&project.manifest, coord, true),
    }
}

pub fn upgrade(project: &JavaProject, coord: &JavaCoord) -> Result<()> {
    // For both Maven and Gradle the upgrade is a re-add: remove + insert.
    let _ = remove(project, coord)?;
    add(project, coord)
}

fn maven_add(pom: &Path, coord: &JavaCoord) -> Result<()> {
    let body = fs::read_to_string(pom).with_context(|| format!("Read {}", pom.display()))?;
    let new_block = maven_dep_block(coord);
    let updated = if let Some(idx) = find_existing_maven_dep(&body, coord) {
        // Replace the existing <dependency> block.
        let (start, end) = idx;
        let mut s = String::new();
        s.push_str(&body[..start]);
        s.push_str(new_block.trim_end_matches('\n'));
        s.push_str(&body[end..]);
        s
    } else if let Some(insert_at) = body.rfind("</dependencies>") {
        // Insert before closing tag.
        let mut s = String::new();
        s.push_str(&body[..insert_at]);
        s.push_str("    ");
        s.push_str(&new_block);
        s.push_str(&body[insert_at..]);
        s
    } else if let Some(end) = body.rfind("</project>") {
        // No <dependencies> at all — synthesize one.
        let mut s = String::new();
        s.push_str(&body[..end]);
        s.push_str("  <dependencies>\n    ");
        s.push_str(&new_block);
        s.push_str("  </dependencies>\n");
        s.push_str(&body[end..]);
        s
    } else {
        return Err(anyhow!("pom.xml has no </project> closing tag"));
    };
    fs::write(pom, updated).with_context(|| format!("Write {}", pom.display()))?;
    Ok(())
}

fn maven_remove(pom: &Path, coord: &JavaCoord) -> Result<bool> {
    let body = fs::read_to_string(pom).with_context(|| format!("Read {}", pom.display()))?;
    if let Some((start, end)) = find_existing_maven_dep(&body, coord) {
        let mut s = String::new();
        // Trim trailing whitespace/newline before the block start so we don't
        // leave a blank line behind.
        let mut tail_start = start;
        while tail_start > 0 {
            let c = body.as_bytes()[tail_start - 1] as char;
            if c == ' ' || c == '\t' {
                tail_start -= 1;
            } else {
                break;
            }
        }
        s.push_str(&body[..tail_start]);
        // Skip a single trailing newline after the block.
        let mut new_end = end;
        if body.as_bytes().get(new_end).copied() == Some(b'\n') {
            new_end += 1;
        }
        s.push_str(&body[new_end..]);
        fs::write(pom, s).with_context(|| format!("Write {}", pom.display()))?;
        return Ok(true);
    }
    Ok(false)
}

fn maven_dep_block(coord: &JavaCoord) -> String {
    let mut s = String::new();
    s.push_str("<dependency>\n");
    s.push_str(&format!("      <groupId>{}</groupId>\n", coord.group));
    s.push_str(&format!("      <artifactId>{}</artifactId>\n", coord.artifact));
    if let Some(v) = &coord.version {
        s.push_str(&format!("      <version>{}</version>\n", v));
    }
    s.push_str("    </dependency>\n");
    s
}

fn find_existing_maven_dep(body: &str, coord: &JavaCoord) -> Option<(usize, usize)> {
    let lower = body.to_ascii_lowercase();
    let needle_g = format!("<groupid>{}</groupid>", coord.group.to_ascii_lowercase());
    let needle_a = format!("<artifactid>{}</artifactid>", coord.artifact.to_ascii_lowercase());
    let mut search_from = 0;
    while let Some(start) = lower[search_from..].find("<dependency>") {
        let abs_start = search_from + start;
        let after = &lower[abs_start..];
        let close = after.find("</dependency>")?;
        let abs_end = abs_start + close + "</dependency>".len();
        let block = &lower[abs_start..abs_end];
        if block.contains(&needle_g) && block.contains(&needle_a) {
            return Some((abs_start, abs_end));
        }
        search_from = abs_end;
    }
    None
}

fn gradle_add(manifest: &Path, coord: &JavaCoord, kotlin: bool) -> Result<()> {
    let body = fs::read_to_string(manifest)
        .with_context(|| format!("Read {}", manifest.display()))?;
    let new_line = gradle_line(coord, kotlin);
    let key = format!("{}:{}", coord.group, coord.artifact);

    // Remove any existing implementation/api line for this coord first.
    let pruned = strip_gradle_line_for(&body, &key);

    // Insert inside `dependencies { ... }`.
    let updated = insert_into_gradle_dependencies(&pruned, &new_line)?;
    fs::write(manifest, updated)
        .with_context(|| format!("Write {}", manifest.display()))?;
    Ok(())
}

fn gradle_remove(manifest: &Path, coord: &JavaCoord, _kotlin: bool) -> Result<bool> {
    let body = fs::read_to_string(manifest)
        .with_context(|| format!("Read {}", manifest.display()))?;
    let key = format!("{}:{}", coord.group, coord.artifact);
    let pruned = strip_gradle_line_for(&body, &key);
    if pruned == body {
        return Ok(false);
    }
    fs::write(manifest, pruned)
        .with_context(|| format!("Write {}", manifest.display()))?;
    Ok(true)
}

fn gradle_line(coord: &JavaCoord, kotlin: bool) -> String {
    let v = coord.version.as_deref().unwrap_or("");
    let coord_str = if v.is_empty() {
        format!("{}:{}", coord.group, coord.artifact)
    } else {
        format!("{}:{}:{}", coord.group, coord.artifact, v)
    };
    if kotlin {
        format!("    implementation(\"{}\")", coord_str)
    } else {
        format!("    implementation '{}'", coord_str)
    }
}

fn strip_gradle_line_for(body: &str, key: &str) -> String {
    body.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            // implementation '<key>...' / implementation "<key>..."
            let starts = trimmed.starts_with("implementation")
                || trimmed.starts_with("api")
                || trimmed.starts_with("compileOnly")
                || trimmed.starts_with("runtimeOnly");
            if !starts {
                return true;
            }
            !line.contains(key)
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if body.ends_with('\n') { "\n" } else { "" }
}

fn insert_into_gradle_dependencies(body: &str, new_line: &str) -> Result<String> {
    if let Some(start) = body.find("dependencies") {
        if let Some(brace) = body[start..].find('{') {
            let block_start = start + brace + 1;
            // Find matching closing brace at column 0.
            let mut depth = 1;
            let mut idx = block_start;
            for (i, c) in body[block_start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            idx = block_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if idx > block_start {
                let mut s = String::new();
                s.push_str(&body[..idx]);
                if !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str(new_line);
                s.push('\n');
                s.push_str(&body[idx..]);
                return Ok(s);
            }
        }
    }
    // No dependencies block — synthesize one at the end.
    let mut s = body.to_string();
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s.push_str("\ndependencies {\n");
    s.push_str(new_line);
    s.push_str("\n}\n");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_at_form() {
        let c = JavaCoord::parse("com.fasterxml.jackson.core:jackson-databind@2.17.0").unwrap();
        assert_eq!(c.group, "com.fasterxml.jackson.core");
        assert_eq!(c.artifact, "jackson-databind");
        assert_eq!(c.version.as_deref(), Some("2.17.0"));
    }

    #[test]
    fn parses_colon_form() {
        let c = JavaCoord::parse("io.javalin:javalin:6.1.6").unwrap();
        assert_eq!(c.group, "io.javalin");
        assert_eq!(c.artifact, "javalin");
        assert_eq!(c.version.as_deref(), Some("6.1.6"));
    }

    #[test]
    fn rejects_bare_artifact() {
        assert!(JavaCoord::parse("requests").is_err());
    }

    #[test]
    fn maven_block_includes_version_only_when_present() {
        let c = JavaCoord::parse("g:a@1.0").unwrap();
        let b = maven_dep_block(&c);
        assert!(b.contains("<version>1.0</version>"));
    }
}
