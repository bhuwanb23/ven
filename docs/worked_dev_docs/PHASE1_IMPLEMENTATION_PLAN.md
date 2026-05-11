# Phase 1: Dependency Graph Foundation - Implementation Plan

**Goal:** Build the core resolver that makes ven intelligent  
**Timeline:** Week 3-4  
**Status:** 📋 PLANNING COMPLETE, READY TO IMPLEMENT

---

## 🎯 Success Criteria

`ven add socket.io` shows full dependency tree, detects conflicts, and previews before installing.

---

## 📐 Architecture Overview

```
ven add express@4.18.2
    ↓
[Phase 1.3] Pre-flight Analysis
    ├─ Parse package spec
    ├─ Get current Node version from ven.toml
    └─ Call DependencyGraph.build()
         ↓
[Phase 1.2] Dependency Graph Builder
    ├─ Call NpmRegistry.fetch_package_metadata()
    │    ↓
    ├─ [Phase 1.1] npm Registry Client
    │    ├─ Check SQLite cache
    │    ├─ If miss: GET registry.npmjs.org/express
    │    ├─ Parse response
    │    └─ Cache in SQLite
    │    ↓
    ├─ Resolve version constraint (^4.0.0 → 4.18.2)
    ├─ Recurse into dependencies
    └─ Build graph
         ↓
[Phase 1.3] Conflict Detection
    ├─ Check compatibility with existing packages
    ├─ Check Node version compatibility at every level
    ├─ Detect duplicate dependencies with different versions
    └─ Generate preview output
         ↓
[User confirms] → npm install → Update ven.toml
```

---

## 🔧 Implementation Order

### Phase 1.1: npm Registry Client (1-2 days)

**File:** `src/core/npm_registry.rs` (NEW)

**What it does:**
- HTTP client for `registry.npmjs.org`
- Caches responses in SQLite
- Handles offline mode
- Rate limiting

**Data Structures:**
```rust
pub struct NpmRegistry {
    cache: SqliteCache,
    client: reqwest::Client,
}

pub struct PackageMetadata {
    pub name: String,
    pub versions: HashMap<String, VersionMetadata>,
    pub dist_tags: HashMap<String, String>, // "latest", "lts"
    pub time: HashMap<String, String>,
}

pub struct VersionMetadata {
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub engines: Option<Engines>,
    pub deprecated: Option<String>,
}

pub struct Engines {
    pub node: Option<String>,
    pub npm: Option<String>,
}
```

**SQLite Schema:**
```sql
CREATE TABLE IF NOT EXISTS packages (
    name TEXT PRIMARY KEY,
    metadata TEXT NOT NULL,  -- JSON blob
    fetched_at INTEGER NOT NULL,  -- Unix timestamp
    expires_at INTEGER NOT NULL   -- Cache expiry
);

CREATE INDEX idx_expires ON packages(expires_at);
```

**Methods:**
```rust
impl NpmRegistry {
    pub fn new() -> Result<Self>;
    
    pub async fn fetch_package_metadata(&self, name: &str) -> Result<PackageMetadata>;
    
    pub async fn fetch_version_metadata(&self, name: &str, version: &str) -> Result<VersionMetadata>;
    
    // Cache management
    fn get_cached(&self, name: &str) -> Option<PackageMetadata>;
    fn set_cached(&self, name: &str, metadata: &PackageMetadata) -> Result<()>;
    fn is_expired(&self, name: &str) -> bool;
    fn cleanup_expired(&self) -> Result<()>;
}
```

**Implementation Details:**

1. **HTTP Client:**
   - Use `reqwest::blocking::get()` for simplicity (async later)
   - URL: `https://registry.npmjs.org/{package_name}`
   - Handle 404 (package not found)
   - Handle rate limiting (429 → retry with backoff)

2. **SQLite Cache:**
   - Database: `~/.ven/cache/registry.db`
   - Cache TTL: 24 hours
   - Auto-cleanup on startup

3. **Offline Mode:**
   - If network fails, use cached data if available
   - Return error if cache miss + offline

**Dependencies to Add:**
```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }  # Already in Cargo.toml
chrono = "0.4"  # For cache expiry timestamps
```

---

### Phase 1.2: Dependency Graph Builder (2-3 days)

**File:** `src/core/resolver.rs` (NEW)

**What it does:**
- Builds complete dependency tree
- Resolves semver constraints
- Detects conflicts
- Checks Node compatibility

**Data Structures:**
```rust
pub struct DependencyGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub conflicts: Vec<Conflict>,
    pub node_version: String,
}

pub struct GraphNode {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub engines: Option<Engines>,
    pub depth: u32,  // How deep in tree (0 = root)
    pub required_by: Vec<String>,  // Reverse lookup
}

pub struct GraphEdge {
    pub from: String,      // "express@4.18.2"
    pub to: String,        // "body-parser@1.20.0"
    pub constraint: String, // "^1.20.0"
}

pub struct Conflict {
    pub package: String,
    pub constraints: Vec<(String, String)>,  // (package, constraint)
    pub reason: String,
}

pub struct Incompatibility {
    pub package: String,
    pub version: String,
    pub required_node: String,
    pub current_node: String,
}
```

**Algorithm:**
```rust
impl DependencyGraph {
    pub async fn build(&mut self, root_package: &str, root_version: &str) -> Result<()> {
        // 1. Fetch root package metadata
        let metadata = self.registry.fetch_package_metadata(root_package).await?;
        
        // 2. Resolve version
        let resolved_version = self.resolve_version(&metadata, root_version)?;
        
        // 3. Add root node
        self.add_node(root_package, &resolved_version, 0)?;
        
        // 4. Recursive dependency fetch
        self.fetch_dependencies(root_package, &resolved_version, 0).await?;
        
        // 5. Detect conflicts
        self.detect_conflicts();
        
        // 6. Check Node compatibility
        self.check_node_compatibility();
        
        Ok(())
    }
    
    async fn fetch_dependencies(&mut self, package: &str, version: &str, depth: u32) -> Result<()> {
        if depth > 20 {
            return Err(anyhow!("Dependency tree too deep"));
        }
        
        // Fetch metadata
        let metadata = self.registry.fetch_version_metadata(package, version).await?;
        
        // Process each dependency
        for (dep_name, dep_constraint) in &metadata.dependencies {
            // Fetch dep metadata
            let dep_metadata = self.registry.fetch_package_metadata(dep_name).await?;
            
            // Resolve version from constraint
            let dep_version = self.resolve_version(&dep_metadata, dep_constraint)?;
            
            // Check if already in graph
            if self.has_node(dep_name) {
                // Check for version conflict
                if self.has_version_conflict(dep_name, &dep_version) {
                    self.conflicts.push(Conflict {
                        package: dep_name.clone(),
                        constraints: vec![...],
                        reason: "Different versions required".to_string(),
                    });
                }
            } else {
                // Add node
                self.add_node(dep_name, &dep_version, depth + 1)?;
                self.add_edge(package, version, dep_name, &dep_version, dep_constraint)?;
                
                // Recurse
                self.fetch_dependencies(dep_name, &dep_version, depth + 1).await?;
            }
        }
        
        Ok(())
    }
    
    fn resolve_version(&self, metadata: &PackageMetadata, constraint: &str) -> Result<String> {
        // Parse constraint: "^4.0.0", "~4.18.0", ">=4.0.0 <5.0.0", "4.18.2", "latest"
        let constraint = semver::VersionReq::parse(constraint)?;
        
        // Get all versions
        let mut versions: Vec<semver::Version> = metadata.versions.keys()
            .filter_map(|v| semver::Version::parse(v).ok())
            .collect();
        
        // Sort descending
        versions.sort_by(|a, b| b.cmp(a));
        
        // Find first version that matches constraint
        versions.into_iter()
            .find(|v| constraint.matches(v))
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow!("No matching version for {}", constraint))
    }
    
    fn detect_conflicts(&mut self) {
        // Group nodes by package name
        let mut packages: HashMap<String, Vec<&GraphNode>> = HashMap::new();
        for node in self.nodes.values() {
            packages.entry(node.name.clone()).or_default().push(node);
        }
        
        // Check for version mismatches
        for (name, nodes) in packages {
            if nodes.len() > 1 {
                let versions: Vec<&str> = nodes.iter().map(|n| n.version.as_str()).collect();
                // Check if all versions are compatible (semver range overlap)
                if !self.versions_compatible(&versions) {
                    self.conflicts.push(Conflict {
                        package: name,
                        constraints: ...,
                        reason: "Incompatible versions in tree".to_string(),
                    });
                }
            }
        }
    }
    
    fn check_node_compatibility(&mut self) -> Vec<Incompatibility> {
        let mut incompatibilities = Vec::new();
        
        for node in self.nodes.values() {
            if let Some(engines) = &node.engines {
                if let Some(node_req) = &engines.node {
                    if !self.node_version_satisfies(&self.node_version, node_req) {
                        incompatibilities.push(Incompatibility {
                            package: node.name.clone(),
                            version: node.version.clone(),
                            required_node: node_req.clone(),
                            current_node: self.node_version.clone(),
                        });
                    }
                }
            }
        }
        
        incompatibilities
    }
}
```

**Dependencies to Add:**
```toml
[dependencies]
semver = "1"  # Already in Cargo.toml
```

---

### Phase 1.3: Rewrite ven add (2-3 days)

**File:** `src/cli/add.rs` (REWRITE)

**What it does:**
- Pre-flight analysis before install
- Shows dependency tree preview
- Detects conflicts
- User confirmation
- Actual install

**Implementation:**
```rust
pub fn cmd_add(package_specs: &[String], skip_check: bool, dry_run: bool) -> Result<()> {
    // 1. Get current config
    let cwd = std::env::current_dir()?;
    let config = load_config(&cwd)?
        .ok_or_else(|| anyhow!("No ven.toml found. Run: ven init"))?;
    
    let node_version = config.runtime.node;
    
    for pkg_spec in package_specs {
        let (name, version) = parse_package_spec(pkg_spec)?;
        
        if !skip_check {
            println!("\n{} Analyzing {}@{}...", "📦".bold(), name, version);
            
            // 2. Build dependency graph
            let mut graph = DependencyGraph::new(node_version.clone());
            graph.build(&name, &version).await?;
            
            // 3. Check for conflicts with existing packages
            let conflicts = graph.check_conflicts_with_existing(&config.packages)?;
            if !conflicts.is_empty() {
                print_conflicts(&conflicts);
                return Err(anyhow!("Conflict detected"));
            }
            
            // 4. Check Node compatibility
            let incompatibilities = graph.check_node_compatibility();
            if !incompatibilities.is_empty() {
                print_node_incompatibilities(&incompatibilities);
                return Err(anyhow!("Node version incompatible"));
            }
            
            // 5. Show preview
            print_install_preview(&graph);
            
            // 6. Confirm (unless dry-run)
            if dry_run {
                println!("\n[DRY RUN] No packages installed");
                return Ok(());
            }
            
            if !confirm_install()? {
                println!("Install cancelled");
                return Ok(());
            }
        }
        
        // 7. Actually install
        println!("\n{} Installing {}@{}...", "📥".bold(), name, version);
        npm_install(&name, &version)?;
        update_ven_toml(&name, &version)?;
        
        println!("{} Installed {}@{}", "✅".green(), name, version);
    }
    
    Ok(())
}

fn print_install_preview(graph: &DependencyGraph) {
    println!("\n{}", "Will install:".bold().cyan());
    
    // Print tree structure
    print_node_tree(graph, "root", 0);
    
    // Summary
    let total_packages = graph.nodes.len();
    let total_size = graph.estimate_download_size();
    
    println!("\n{} {} packages", "📦".bold(), total_packages);
    println!("{} {:.1} MB", "💾".bold(), total_size);
}

fn print_node_tree(graph: &DependencyGraph, package: &str, depth: u32) {
    let indent = "  ".repeat(depth as usize);
    let connector = if depth == 0 { "" } else { "├─ " };
    
    if let Some(node) = graph.nodes.get(package) {
        println!("{}{}{}@{}", indent, connector, node.name.bold(), node.version);
        
        // Print children
        for child in graph.get_children(package) {
            print_node_tree(graph, &child.name, depth + 1);
        }
    }
}
```

**New Command Flags:**
```rust
Commands::Add {
    packages: Vec<String>,
    skip_check: bool,      // Existing
    dry_run: bool,         // NEW: Preview only
    verbose: bool,         // NEW: Show full dependency tree
}
```

**Output Example:**
```
$ ven add socket.io@4.7

📦 Analyzing socket.io@4.7.5...

✓ Compatible with Node 20.11.0
✓ Compatible with express@4.18.2

Will install 12 packages:
  socket.io@4.7.5
  ├─ engine.io@6.5.4
  │  ├─ ws@8.16.0 ⚠ DUPLICATE (express uses ws@8.14.0)
  │  ├─ base64id@2.0.0
  │  └─ cookie@0.6.0
  ├─ socket.io-adapter@2.5.4
  └─ socket.io-parser@4.2.4

📦 12 packages
💾 2.3 MB
⏱ Estimated time: 3s

Proceed? [Y/n]
```

---

## 📁 New Files to Create

1. **`src/core/npm_registry.rs`** (~300 lines)
   - NpmRegistry client
   - SQLite cache
   - HTTP requests
   - Offline mode

2. **`src/core/resolver.rs`** (~500 lines)
   - DependencyGraph
   - Recursive fetcher
   - Conflict detector
   - Node compatibility checker

3. **`src/core/mod.rs`** (UPDATE)
   - Export new modules

---

## 📝 Modified Files

1. **`src/cli/add.rs`** (REWRITE)
   - Add pre-flight analysis
   - Preview output
   - Dry-run mode

2. **`Cargo.toml`** (UPDATE)
   - Add chrono for timestamps

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_resolve_version_constraint() {
    assert_eq!(resolve_version("^4.0.0", &["4.0.0", "4.1.0", "5.0.0"]), "4.1.0");
}

#[test]
fn test_detect_conflict() {
    // Package A needs ws@^8.0.0
    // Package B needs ws@^7.0.0
    // Should detect conflict
}

#[test]
fn test_node_compatibility() {
    // Package requires node >= 18
    // Current node = 16
    // Should fail
}
```

### Integration Tests
```bash
# Test 1: Simple install with preview
ven add express --dry-run

# Test 2: Complex dependency tree
ven add socket.io --verbose

# Test 3: Conflict detection
ven init  # with express@4.18.2
ven add webpack  # may have conflicting deps

# Test 4: Node incompatibility
# Set Node 16 in ven.toml
ven add package-that-needs-node-18  # should warn

# Test 5: Offline mode
# Disable network
ven add express  # should use cache
```

---

## ⚡ Performance Considerations

1. **Caching:** SQLite cache avoids repeated HTTP requests
   - Cache TTL: 24 hours
   - Lazy loading: Only fetch what's needed
   - Parallel fetches: Use async for concurrent requests

2. **Depth Limit:** Max 20 levels deep (prevents infinite recursion)

3. **Rate Limiting:** 
   - npm registry allows ~100 req/min
   - Add delays between requests
   - Batch requests when possible

4. **Memory:** Graph stored in memory
   - Large trees (500+ packages) may use 10-20MB RAM
   - Acceptable for CLI tool

---

## 🚀 Implementation Steps

**Day 1-2:**
1. Create `npm_registry.rs`
2. Implement HTTP client
3. Add SQLite cache
4. Write unit tests

**Day 3-4:**
1. Create `resolver.rs`
2. Build DependencyGraph structure
3. Implement recursive fetcher
4. Add semver resolution

**Day 5-6:**
1. Implement conflict detection
2. Add Node compatibility checker
3. Write integration tests

**Day 7-8:**
1. Rewrite `add.rs` with pre-flight
2. Add preview output
3. Implement dry-run mode
4. Test with real packages

**Day 9-10:**
1. Polish output formatting
2. Add error messages
3. Performance optimization
4. Documentation

---

## 🎯 Success Metrics

- ✅ `ven add express` shows full dependency tree
- ✅ Conflicts detected before install
- ✅ Node compatibility checked at every level
- ✅ Dry-run mode works
- ✅ Offline mode uses cache
- ✅ Install preview shows package count + size
- ✅ User can confirm/cancel before install

---

## 📊 Phase 1 Completion Checklist

- [ ] npm Registry Client implemented
- [ ] SQLite cache working
- [ ] Dependency Graph structure complete
- [ ] Recursive dependency fetcher working
- [ ] Semver constraint resolver implemented
- [ ] Conflict detector working
- [ ] Node compatibility checker working
- [ ] ven add rewritten with pre-flight
- [ ] Preview output formatted
- [ ] Dry-run mode working
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Tested with real packages

---

**Ready to start implementing?** I'll begin with Phase 1.1 (npm Registry Client) and work through systematically.
