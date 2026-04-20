use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use colored::Colorize;
use crate::core::npm_registry::{NpmRegistry, PackageMetadata, VersionMetadata};

/// Dependency graph for analyzing package relationships
pub struct DependencyGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub conflicts: Vec<Conflict>,
    pub incompatibilities: Vec<NodeIncompatibility>,
    pub node_version: String,
    registry: NpmRegistry,
}

/// A node in the dependency graph (a specific package version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub engines: Option<String>,  // Node version requirement
    pub depth: u32,
    pub required_by: Vec<String>,
    pub deprecated: Option<String>,
}

/// An edge in the dependency graph (a dependency relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,       // "express@4.18.2"
    pub to: String,         // "body-parser@1.20.0"
    pub constraint: String, // "^1.20.0"
}

/// A version conflict detected in the graph
#[derive(Debug, Clone)]
pub struct Conflict {
    pub package: String,
    pub constraints: Vec<(String, String)>, // (requirer, constraint)
    pub versions: Vec<String>,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone)]
pub enum ConflictSeverity {
    Warning, // Different versions but can coexist
    Error,   // Incompatible versions
}

/// Node.js version incompatibility
#[derive(Debug, Clone)]
pub struct NodeIncompatibility {
    pub package: String,
    pub version: String,
    pub required_node: String,
    pub current_node: String,
}

/// Preview information for installation
pub struct InstallPreview {
    pub total_packages: usize,
    pub total_size_bytes: u64,
    pub new_packages: Vec<String>,
    pub duplicate_packages: Vec<String>,
    pub warnings: Vec<String>,
}

impl DependencyGraph {
    pub fn new(node_version: String) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            conflicts: Vec::new(),
            incompatibilities: Vec::new(),
            node_version,
            registry: NpmRegistry::new().expect("Failed to initialize npm registry client"),
        }
    }

    /// Build complete dependency graph for a package
    pub async fn build(&mut self, root_package: &str, root_version: &str) -> Result<()> {
        println!("  {} Building dependency graph...", "🔍".cyan());

        // Fetch root package metadata
        let metadata = self.registry.fetch_package_metadata(root_package)?;

        // Resolve version (handle "latest", "^4.0.0", etc.)
        let resolved_version = self.resolve_version(&metadata, root_version)?;

        // Add root node
        self.add_node(root_package, &resolved_version, &metadata, 0, None)?;

        // Recursively fetch dependencies
        self.fetch_dependencies(root_package, &resolved_version, 0).await?;

        // Analyze graph
        self.detect_conflicts();
        self.check_node_compatibility();

        println!("  {} Graph built: {} packages, {} edges", 
            "✓".green(), 
            self.nodes.len(), 
            self.edges.len()
        );

        Ok(())
    }

    /// Recursively fetch all dependencies
    async fn fetch_dependencies(&mut self, package: &str, version: &str, depth: u32) -> Result<()> {
        if depth > 20 {
            return Err(anyhow!("Dependency tree too deep (>{})", depth));
        }

        // Fetch version metadata
        let version_meta = match self.registry.fetch_version_metadata(package, version) {
            Ok(meta) => meta,
            Err(e) => {
                eprintln!("  {} Warning: Failed to fetch {}@{}: {}", "⚠".yellow(), package, version, e);
                return Ok(()); // Continue with other deps
            }
        };

        // Get dependencies (skip devDependencies for now - those are for development)
        let dependencies = version_meta.dependencies.unwrap_or_default();

        for (dep_name, dep_constraint) in &dependencies {
            // Fetch dependency metadata
            let dep_metadata = match self.registry.fetch_package_metadata(dep_name) {
                Ok(meta) => meta,
                Err(e) => {
                    eprintln!("  {} Warning: Failed to fetch {}: {}", "⚠".yellow(), dep_name, e);
                    continue;
                }
            };

            // Resolve version from constraint
            let dep_version = match self.resolve_version(&dep_metadata, dep_constraint) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("  {} Warning: Cannot resolve {} for {}: {}", 
                        "⚠".yellow(), dep_constraint, dep_name, e);
                    continue;
                }
            };

            let edge_from = format!("{}@{}", package, version);
            let edge_to = format!("{}@{}", dep_name, dep_version);

            // Add edge
            self.edges.push(GraphEdge {
                from: edge_from.clone(),
                to: edge_to.clone(),
                constraint: dep_constraint.clone(),
            });

            // Check if node already exists
            if self.nodes.contains_key(dep_name) {
                // Check for version conflict
                let existing = self.nodes.get(dep_name).unwrap();
                if existing.version != dep_version {
                    // Version mismatch - add to required_by but flag as potential conflict
                    if let Some(node) = self.nodes.get_mut(dep_name) {
                        node.required_by.push(edge_from);
                    }
                }
            } else {
                // Add new node
                self.add_node(dep_name, &dep_version, &dep_metadata, depth + 1, Some(&edge_from))?;

                // Recurse into this dependency's dependencies
                Box::pin(self.fetch_dependencies(dep_name, &dep_version, depth + 1)).await?;
            }
        }

        Ok(())
    }

    /// Add a node to the graph
    fn add_node(
        &mut self,
        name: &str,
        version: &str,
        metadata: &PackageMetadata,
        depth: u32,
        required_by: Option<&str>,
    ) -> Result<()> {
        // Get version-specific metadata
        let version_meta = metadata.versions.get(version);

        let dependencies = version_meta
            .and_then(|v| v.dependencies.clone())
            .unwrap_or_default();

        let engines = version_meta
            .and_then(|v| v.engines.clone())
            .and_then(|e| e.node);

        let deprecated = version_meta.and_then(|v| v.deprecated.clone());

        let node = GraphNode {
            name: name.to_string(),
            version: version.to_string(),
            dependencies,
            engines,
            depth,
            required_by: required_by.map(|s| vec![s.to_string()]).unwrap_or_default(),
            deprecated,
        };

        self.nodes.insert(name.to_string(), node);
        Ok(())
    }

    /// Resolve version constraint to specific version
    fn resolve_version(&self, metadata: &PackageMetadata, constraint: &str) -> Result<String> {
        // Handle special tags
        if constraint == "latest" {
            return metadata.dist_tags.get("latest")
                .cloned()
                .ok_or_else(|| anyhow!("No 'latest' tag found"));
        }

        if constraint == "lts" {
            // Find latest LTS version (even major number)
            let mut versions: Vec<semver::Version> = metadata.versions.keys()
                .filter_map(|v| semver::Version::parse(v).ok())
                .filter(|v| v.major % 2 == 0) // LTS = even major
                .collect();
            
            versions.sort_by(|a, b| b.cmp(a));
            return versions.first()
                .map(|v| v.to_string())
                .ok_or_else(|| anyhow!("No LTS versions found"));
        }

        // Parse as semver constraint
        match semver::VersionReq::parse(constraint) {
            Ok(req) => {
                // Find highest version matching constraint
                let mut versions: Vec<semver::Version> = metadata.versions.keys()
                    .filter_map(|v| semver::Version::parse(v).ok())
                    .filter(|v| req.matches(v))
                    .collect();

                versions.sort_by(|a, b| b.cmp(a));

                versions.first()
                    .map(|v| v.to_string())
                    .ok_or_else(|| anyhow!("No version matches '{}'", constraint))
            }
            Err(_) => {
                // Try parsing as exact version
                semver::Version::parse(constraint)
                    .map(|v| v.to_string())
                    .or_else(|_| {
                        // Try as major version (e.g., "4" -> "4.x.x")
                        if let Ok(major) = constraint.parse::<u64>() {
                            let mut versions: Vec<semver::Version> = metadata.versions.keys()
                                .filter_map(|v| semver::Version::parse(v).ok())
                                .filter(|v| v.major == major)
                                .collect();
                            
                            versions.sort_by(|a, b| b.cmp(a));
                            return versions.first()
                                .map(|v| v.to_string())
                                .ok_or_else(|| anyhow!("No version {}.* found", major));
                        }
                        Err(anyhow!("Invalid version constraint: {}", constraint))
                    })
            }
        }
    }

    /// Detect version conflicts in the graph
    fn detect_conflicts(&mut self) {
        // Group edges by target package
        let mut package_constraints: HashMap<String, Vec<(String, String)>> = HashMap::new();
        
        for edge in &self.edges {
            let dep_name = edge.to.split('@').next().unwrap_or("");
            package_constraints
                .entry(dep_name.to_string())
                .or_default()
                .push((edge.from.clone(), edge.constraint.clone()));
        }

        // Check for conflicts
        for (package, constraints) in package_constraints {
            if constraints.len() > 1 {
                // Multiple packages depend on this one with different constraints
                self.conflicts.push(Conflict {
                    package,
                    constraints,
                    versions: vec![], // Will be populated during analysis
                    severity: ConflictSeverity::Warning,
                });
            }
        }
    }

    /// Check Node.js compatibility at every level
    fn check_node_compatibility(&mut self) {
        for (name, node) in &self.nodes {
            if let Some(ref engine_req) = node.engines {
                if !DependencyGraph::node_version_satisfies(&self.node_version, engine_req) {
                    self.incompatibilities.push(NodeIncompatibility {
                        package: name.clone(),
                        version: node.version.clone(),
                        required_node: engine_req.clone(),
                        current_node: self.node_version.clone(),
                    });
                }
            }
        }
    }

    /// Check if Node version satisfies requirement
    fn node_version_satisfies(node_version: &str, requirement: &str) -> bool {
        let req = requirement.trim();
        if req == "*" || req.is_empty() {
            return true;
        }

        // Parse current Node major version
        let node_major = node_version.split('.')
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0);

        // Extract minimum version from requirement
        let min_ver: String = req.chars()
            .skip_while(|c| !c.is_ascii_digit())
            .collect();

        let min_major = min_ver.split('.')
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0);

        node_major >= min_major
    }

    /// Generate install preview
    pub fn generate_preview(&self) -> InstallPreview {
        let new_packages: Vec<String> = self.nodes.keys().cloned().collect();
        
        let duplicate_packages: Vec<String> = self.conflicts.iter()
            .map(|c| c.package.clone())
            .collect();

        let warnings: Vec<String> = self.incompatibilities.iter()
            .map(|i| format!("{}@{} requires Node {}", i.package, i.version, i.required_node))
            .collect();

        // Estimate size (average 50KB per package - rough estimate)
        let total_size_bytes = self.nodes.len() as u64 * 50 * 1024;

        InstallPreview {
            total_packages: self.nodes.len(),
            total_size_bytes,
            new_packages,
            duplicate_packages,
            warnings,
        }
    }

    /// Check compatibility with existing packages
    pub fn check_existing_compatibility(
        &self,
        existing_packages: &HashMap<String, String>,
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for (existing_name, existing_version) in existing_packages {
            if let Some(new_node) = self.nodes.get(existing_name) {
                if new_node.version != *existing_version {
                    conflicts.push(Conflict {
                        package: existing_name.clone(),
                        constraints: vec![
                            ("ven.toml".to_string(), existing_version.clone()),
                            ("new dependency".to_string(), new_node.version.clone()),
                        ],
                        versions: vec![existing_version.clone(), new_node.version.clone()],
                        severity: ConflictSeverity::Warning,
                    });
                }
            }
        }

        conflicts
    }

    /// Print dependency tree
    pub fn print_tree(&self) {
        // Find root node (depth 0)
        let root = self.nodes.values()
            .find(|n| n.depth == 0);

        if let Some(root_node) = root {
            self.print_node(root_node, 0, true);
        }
    }

    fn print_node(&self, node: &GraphNode, depth: u32, is_last: bool) {
        let indent = "  ".repeat(depth as usize);
        let connector = if depth == 0 {
            "".to_string()
        } else if is_last {
            "└─ ".to_string()
        } else {
            "├─ ".to_string()
        };

        let name_version = format!("{}@{}", node.name, node.version);
        
        // Check if deprecated
        let deprecated_marker = if node.deprecated.is_some() {
            " ⚠️ DEPRECATED".yellow().to_string()
        } else {
            "".to_string()
        };

        // Check if has conflicts
        let conflict_marker = if self.conflicts.iter().any(|c| c.package == node.name) {
            " ⚠ CONFLICT".red().to_string()
        } else {
            "".to_string()
        };

        if depth == 0 {
            println!("{}{}", name_version.bold().cyan(), deprecated_marker);
        } else {
            println!("{}{}{}{}{}", indent, connector, name_version, deprecated_marker, conflict_marker);
        }

        // Print children
        let children: Vec<&GraphEdge> = self.edges.iter()
            .filter(|e| e.from == format!("{}@{}", node.name, node.version))
            .collect();

        for (i, edge) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            let child_name = edge.to.split('@').next().unwrap_or("");
            
            if let Some(child_node) = self.nodes.get(child_name) {
                self.print_node(child_node, depth + 1, is_last_child);
            }
        }
    }
}
