# ven add

Install npm packages with automatic Node.js compatibility checking.

## Overview

The `add` command is ven's **intelligent package installer** that prevents breaking changes by checking package compatibility **before** installation.

Unlike `npm install` (which installs first and breaks later), `ven add`:

- ✅ Fetches package metadata from npm registry
- ✅ Checks Node.js engine constraints
- ✅ Finds the highest compatible version
- ✅ Installs the safe version
- ✅ Updates `ven.toml` automatically

## Usage

### Basic Installation

```bash
ven add <package>[@version]
```

### Examples

#### Auto-Compatible Version

```bash
ven add express
```

**Output:**
```
→ Checking express against Node 20.11.0...
  ✓ express@4.21.2 — compatible with Node 20.11.0
[DOWNLOAD] Installing express@4.21.2...

added 68 packages in 2s
[OK] Installed express@4.21.2
  ✓ Updated ven.toml
```

#### Pin Specific Version

```bash
ven add express@4.18.2
```

**Output:**
```
→ Checking express@4.18.2 against Node 20.11.0...
  ✓ express@4.18.2 — compatible with Node 20.11.0
[DOWNLOAD] Installing express@4.18.2...
[OK] Installed express@4.18.2
  ✓ Updated ven.toml
```

#### Skip Compatibility Check

```bash
ven add express --skip-check
```

**Output:**
```
→ Skipping compatibility check for express...
[DOWNLOAD] Installing express@latest...

added 68 packages in 2s
[OK] Installed express@latest
  ✓ Updated ven.toml
```

---

## Command Reference

### Syntax

```bash
ven add [OPTIONS] <package>
```

### Arguments

| Argument | Required | Description | Example |
|----------|----------|-------------|---------|
| `package` | Yes | Package name with optional version | `express`, `express@4.18.2` |

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--skip-check` | Skip Node.js compatibility check | `false` |

---

## How It Works

### Compatibility Check Process

```
1. Read current Node version from ven.toml
   ↓
2. Fetch package metadata from npm registry
   ↓
3. Check engine constraints for each version
   ↓
4. Find highest compatible version
   ↓
5. Install via npm
   ↓
6. Update ven.toml
```

### Example Walkthrough

**Project Setup:**
```toml
# ven.toml
[runtime]
node = "18.15.0"
```

**Command:**
```bash
ven add next
```

**Process:**
1. Read Node version: `18.15.0`
2. Fetch: `https://registry.npmjs.org/next`
3. Check Next.js 14.1.0: requires Node >= 18.17.0 ❌
4. Check Next.js 13.5.6: requires Node >= 18.17.0 ❌
5. Check Next.js 13.4.0: requires Node >= 16.8.0 ✅
6. Install: `npm install next@13.4.0`
7. Update ven.toml: `next = "13.4.0"`

**Output:**
```
→ Checking next against Node 18.15.0...
  ✓ next@13.4.0 — compatible with Node 18.15.0
[DOWNLOAD] Installing next@13.4.0...
[OK] Installed next@13.4.0
  ✓ Updated ven.toml
```

---

## Package Spec Formats

### Supported Formats

| Format | Example | Behavior |
|--------|---------|----------|
| Package only | `express` | Find best compatible |
| With version | `express@4.18.2` | Install exact version |
| Scoped package | `@types/node` | Works with `@` prefix |
| Scoped + version | `@types/node@20.11.0` | Exact scoped version |

### Examples

```bash
# Regular package
ven add express

# Pinned version
ven add react@18.2.0

# Scoped package
ven add @types/express

# Scoped + version
ven add @testing-library/react@14.1.2
```

---

## ven.toml Updates

### Before

```toml
[runtime]
node = "20.11.0"

[packages]
```

### After `ven add express`

```toml
[runtime]
node = "20.11.0"

[packages]
express = "4.21.2"
```

### After `ven add typescript@5.3.0`

```toml
[runtime]
node = "20.11.0"

[packages]
express = "4.21.2"
typescript = "5.3.0"
```

---

## Compatibility Checking

### Engine Constraints

Packages can specify Node.js version requirements:

```json
{
  "name": "express",
  "engines": {
    "node": ">= 0.10.0"
  }
}
```

**ven checks:**
- Current Node version: `20.11.0`
- Package requires: `>= 0.10.0`
- **Result**: ✅ Compatible

### Supported Engine Formats

| Format | Example | Meaning |
|--------|---------|---------|
| Minimum version | `>= 18.0.0` | Node 18.0.0 or higher |
| Range | `>=14 <20` | Node 14-19 |
| Exact | `18.15.0` | Exactly this version |
| Wildcard | `*` | Any version |
| Caret | `^18` | Compatible with 18.x.x |

### Simplified Checking (Phase 1)

For Phase 1, ven uses **major version comparison**:

```rust
fn node_version_satisfies(node_ver: &str, requirement: &str) -> bool {
    // Parse: "20.11.0" → 20
    let node_major = node_ver.split('.').next()...;
    
    // Parse: ">= 18.0.0" → 18
    let min_major = requirement.trim()...;
    
    // Check: 20 >= 18 → true
    node_major >= min_major
}
```

**Future Enhancement**: Full semver parsing in Phase 2.

---

## Version Resolution Strategy

### 1. Try Latest First

```rust
if let Some(latest) = info.dist_tags.get("latest") {
    if is_compatible(info, latest, node_version) {
        return Some(latest.clone());  // Use latest if compatible
    }
}
```

### 2. Fall Back to Highest Compatible

```rust
// Sort all versions descending
let mut versions: Vec<&String> = info.versions.keys().collect();
versions.sort_by(|a, b| semver_cmp(b, a));

// Find first compatible
versions.into_iter()
    .find(|v| is_compatible(info, v, node_version))
```

### 3. No Compatible Version

```bash
ven add some-very-new-package
```

**Output:**
```
→ Checking some-very-new-package against Node 16.20.0...
Error: No compatible version of some-very-new-package found for Node 16.20.0
```

**Solution:**
```bash
# Upgrade Node.js
ven install node 20

# Try again
ven add some-very-new-package
```

---

## Error Handling

### Package Not Found

```bash
ven add non-existent-package
```

**Output:**
```
Error: Package 'non-existent-package' not found on npm
```

### No Compatible Version

```bash
ven add express
# With Node 10
```

**Output:**
```
→ Checking express against Node 10.24.1...
Error: No compatible version of express found for Node 10.24.1
```

### No ven.toml

```bash
cd /tmp/
ven add express
```

**Output:**
```
Error: No ven.toml found. Run: ven init
```

### npm Not Found

```bash
ven add express
```

**Output:**
```
Error: npm not found. Is Node installed and active?
```

---

## Skip Compatibility Check

### When to Use `--skip-check`

- Testing with incompatible versions
- Working with packages missing engine constraints
- Forcing installation despite warnings
- Development/experimental scenarios

### Warning

```bash
ven add express --skip-check
```

**Risks:**
- Package may not work correctly
- Runtime errors possible
- No guarantee of compatibility
- May break on certain Node features

### Example

```bash
# Install latest even if incompatible
ven add super-new-package --skip-check

# Output
→ Skipping compatibility check for super-new-package...
[DOWNLOAD] Installing super-new-package@latest...
[OK] Installed super-new-package@latest
  ✓ Updated ven.toml
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/add.rs`](../../src/cli/add.rs) (86 lines)
- **Package logic**: [`src/core/packages.rs`](../../src/core/packages.rs)

### Key Functions

```rust
// CLI layer
cmd_add(package_spec, skip_check)
update_ven_toml_package(pkg, version)

// Core package management
fetch_npm_info(package)              // Fetch from registry
find_compatible_version(info, node)  // Find best version
npm_install(package, version)        // Run npm install
```

### npm Registry API

```rust
// Fetch package metadata
let url = format!("https://registry.npmjs.org/{}", package);
let response = reqwest::blocking::get(&url)?;
let info: NpmPackageInfo = response.json()?;
```

**Response structure:**
```json
{
  "name": "express",
  "dist-tags": {
    "latest": "4.21.2",
    "next": "5.0.0-beta.1"
  },
  "versions": {
    "4.21.2": {
      "engines": {
        "node": ">= 0.10.0"
      }
    }
  }
}
```

### Dependencies

```toml
reqwest 0.12      # HTTP client
serde 1           # JSON deserialization
colored 2         # Terminal colors
anyhow 1          # Error handling
```

---

## Use Cases

### 1. New Project Setup

```bash
ven init --template
ven add express
ven add typescript --skip-check
ven add @types/express
```

### 2. Upgrade Package

```bash
# Check for updates
ven upgrade express

# Apply upgrade
ven upgrade express --apply
```

### 3. Install Dev Dependencies

```bash
# Currently requires manual approach
npm install --save-dev jest eslint

# Then update ven.toml manually or use:
ven add jest  # Will track in packages section
```

### 4. Monorepo Packages

```bash
# Frontend
cd monorepo/frontend/
ven add react
ven add vite

# Backend
cd ../backend/
ven add express
ven add prisma
```

---

## Troubleshooting

### Installation Fails

**Problem**: npm install error

**Solution:**
```bash
# Check npm works
npm --version

# Clear npm cache
npm cache clean --force

# Try again
ven add express
```

### Wrong Version Installed

**Problem**: ven picked wrong version

**Solution:**
```bash
# Pin exact version
ven add express@4.18.2

# Or check what was resolved
cat ven.toml
```

### Compatibility Check Too Strict

**Problem**: ven rejects package unnecessarily

**Solution:**
```bash
# Skip check for this package
ven add package --skip-check

# Or upgrade Node.js
ven install node 20
```

---

## Related Commands

- [`ven remove`](remove.md) - Remove packages safely
- [`ven upgrade`](upgrade.md) - Upgrade packages
- [`ven init`](init.md) - Create project with packages

---

## Next Steps

After adding packages:

```bash
# View project status
ven status

# Check installed versions
ven list

# Remove a package
ven remove express
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
