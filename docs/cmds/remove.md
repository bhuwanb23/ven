# ven remove

Safely remove npm packages with dependency analysis and interactive warnings.

## Overview

The `remove` command prevents accidental breaking changes by analyzing the dependency graph **before** removal.

Unlike `npm uninstall` (which removes without warning), `ven remove`:

- ✅ Parses `package-lock.json`
- ✅ Finds all packages that depend on the target
- ✅ Displays clear warnings
- ✅ Requires interactive confirmation
- ✅ Supports `--force` flag to bypass checks

## Usage

### Basic Removal

```bash
ven remove <package>
```

### Examples

#### Remove Package (Safe)

```bash
ven remove accepts
```

**Output:**
```
  Warning: 2 packages depend on accepts:
    • express 4.18.2  requires  accepts ~1.3.8
    • koa 2.14.0      requires  accepts ^1.3

  Removing accepts may break these packages.
  Remove anyway? [y/N]: n
  Cancelled.
```

#### Force Removal

```bash
ven remove lodash --force
```

**Output:**
```
✓ Removed lodash
```

#### Remove Package (No Dependents)

```bash
ven remove unused-package
```

**Output:**
```
✓ Removed unused-package
```

---

## Command Reference

### Syntax

```bash
ven remove [OPTIONS] <package>
```

### Arguments

| Argument | Required | Description | Example |
|----------|----------|-------------|---------|
| `package` | Yes | Package name to remove | `express`, `lodash` |

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--force` | Skip dependency checking | `false` |

---

## How It Works

### Dependency Analysis Process

```
1. Read package-lock.json
   ↓
2. Parse all installed packages
   ↓
3. Check each package's dependencies
   ↓
4. Find packages that require target
   ↓
5. Display warnings if dependents found
   ↓
6. Prompt for confirmation
   ↓
7. Remove if confirmed (or force)
```

### Example Walkthrough

**Dependency Graph:**
```
express@4.18.2
├── accepts@1.3.8     ← Target
├── body-parser@1.20.0
└── cookie@0.5.0

koa@2.14.0
└── accepts@1.3.8     ← Target
```

**Command:**
```bash
ven remove accepts
```

**Process:**
1. Read `package-lock.json`
2. Scan all packages
3. Found: `express` depends on `accepts ~1.3.8`
4. Found: `koa` depends on `accepts ^1.3`
5. Display warning with both dependents
6. Wait for user confirmation

---

## Interactive Confirmation

### Default Behavior (Safe Mode)

```bash
ven remove accepts
```

**Prompts:**
```
  Warning: 2 packages depend on accepts:
    • express 4.18.2  requires  accepts ~1.3.8
    • koa 2.14.0      requires  accepts ^1.3

  Removing accepts may break these packages.
  Remove anyway? [y/N]:
```

**Responses:**
- `y` or `Y`: Proceed with removal
- `n`, `N`, or `Enter`: Cancel
- Any other input: Cancel

### Force Mode

```bash
ven remove accepts --force
```

**Behavior:**
- Skips dependency analysis
- No confirmation prompt
- Removes immediately
- ⚠️ **Use with caution**

---

## Dependency Detection

### How It Works

```rust
pub fn find_dependents(package: &str) -> Result<Vec<(String, String)>> {
    let lock_path = std::env::current_dir()?.join("package-lock.json");

    if !lock_path.exists() {
        return Ok(vec![]); // No lock file = cannot check
    }

    let content = std::fs::read_to_string(&lock_path)?;
    let lock: serde_json::Value = serde_json::from_str(&content)?;

    let mut dependents = Vec::new();

    // Walk packages in lock file
    if let Some(packages) = lock["packages"].as_object() {
        for (name, info) in packages {
            if name.is_empty() { continue; } // Skip root
            
            // Check if this package depends on target
            if let Some(deps) = info["dependencies"].as_object() {
                if deps.contains_key(package) {
                    let clean_name = name.trim_start_matches("node_modules/");
                    let version = info["version"].as_str().unwrap_or("").to_string();
                    dependents.push((clean_name.to_string(), version));
                }
            }
        }
    }

    Ok(dependents)
}
```

### package-lock.json Format

**Example structure:**
```json
{
  "name": "my-project",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {
    "": {
      "name": "my-project",
      "dependencies": {
        "express": "^4.18.2"
      }
    },
    "node_modules/express": {
      "version": "4.18.2",
      "dependencies": {
        "accepts": "~1.3.8",
        "body-parser": "1.20.1"
      }
    },
    "node_modules/accepts": {
      "version": "1.3.8",
      "dependencies": {}
    }
  }
}
```

**ven scans:**
- `node_modules/express` → has `accepts` in dependencies
- `node_modules/koa` → has `accepts` in dependencies
- Reports both as dependents

---

## Warning Display

### Format

```
  Warning: <count> packages depend on <package>:
    • <dependent1> <version1>  requires  <package> <constraint1>
    • <dependent2> <version2>  requires  <package> <constraint2>

  Removing <package> may break these packages.
  Remove anyway? [y/N]:
```

### Real Example

```bash
ven remove mime-types
```

**Output:**
```
  Warning: 4 packages depend on mime-types:
    • express 4.18.2      requires  mime-types ~2.1.34
    • type-is 1.6.18      requires  mime-types ~2.1.24
    • accepts 1.3.8       requires  mime-types ~2.1.34
    • send 0.18.0         requires  mime-types ~2.1.35

  Removing mime-types may break these packages.
  Remove anyway? [y/N]:
```

---

## Edge Cases

### No package-lock.json

**Situation**: Project doesn't have lock file yet

**Behavior:**
```bash
ven remove lodash
```

**Output:**
```
✓ Removed lodash
```

**Reason:** No lock file = cannot analyze = allow removal

**Solution:** Run `npm install` first to generate lock file.

### Package Not Installed

**Situation**: Trying to remove non-existent package

**Output:**
```
Error: npm uninstall failed for nonexistent
```

**Solution:** Verify package name with `npm list`

### Global Packages

**Note**: `ven remove` only works for **local** project packages.

**Global removal:**
```bash
# Use npm directly for global packages
npm uninstall -g package-name
```

---

## Use Cases

### 1. Safe Cleanup

```bash
# Check dependencies first
ven remove unused-lib

# See what depends on it
# If nothing, safe to remove
# If dependencies, reconsider
```

### 2. Force Removal (Known Safe)

```bash
# You know it's safe
ven remove old-package --force
```

### 3. Dependency Audit

```bash
# See what depends on a package
ven remove accepts

# Read the warning to understand usage
# Cancel with 'n' to keep it
```

### 4. Breaking Change Prevention

```bash
# Team member suggests removing lodash
ven remove lodash

# Discover 15 packages depend on it
# Decide to keep it or refactor first
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/remove.rs`](../../src/cli/remove.rs) (43 lines)
- **Dependency analysis**: [`src/core/packages.rs`](../../src/core/packages.rs)

### Key Functions

```rust
// CLI layer
cmd_remove(package, force)

// Core package management
find_dependents(package)     // Analyze dependency graph
npm_uninstall(package)       // Run npm uninstall
```

### Dependencies

```rust
use std::io::{self, BufRead};  // Interactive input
use colored::Colorize;         // Terminal colors
use serde_json;                // Parse lock file
```

---

## Comparison with npm

### npm uninstall

```bash
npm uninstall accepts
```

**Behavior:**
- Removes immediately
- No dependency warning
- May break other packages
- Silent failure possible

### ven remove

```bash
ven remove accepts
```

**Behavior:**
- Analyzes dependencies first
- Shows clear warnings
- Requires confirmation
- Prevents accidental breaks

---

## Best Practices

### 1. Always Check First

```bash
# Don't use --force unless sure
ven remove package-name

# Read the warnings carefully
# Understand impact before confirming
```

### 2. Use in CI/CD

```bash
# Scripted removal with force
ven remove dev-package --force
```

### 3. Team Communication

```bash
# Before removing shared dependency
ven remove shared-lib

# Share warning output with team
# Discuss impact before proceeding
```

### 4. Refactoring Workflow

```bash
# 1. Check what depends on it
ven remove old-package

# 2. Plan migration
# 3. Update dependents
# 4. Remove old package
ven remove old-package --force
```

---

## Troubleshooting

### Warning Appears But Shouldn't

**Problem**: Package shows dependents but they're optional

**Solution:**
```bash
# Verify if really needed
npm ls package-name

# If optional, force remove
ven remove package-name --force
```

### Too Many Dependents Listed

**Problem**: Warning is overwhelming

**Solution:**
```bash
# Check lock file manually
cat package-lock.json | grep -A 5 "package-name"

# Or use npm
npm ls package-name
```

### Cannot Remove Package

**Problem**: Permission denied

**Solution:**
```bash
# Fix permissions
sudo chown -R $USER node_modules/

# Or use force
ven remove package --force
```

---

## Related Commands

- [`ven add`](add.md) - Add packages safely
- [`ven upgrade`](upgrade.md) - Upgrade packages
- [`ven status`](status.md) - Check project config

---

## Next Steps

After removing packages:

```bash
# Verify project still works
npm test

# Check what's left
ven status

# Update lock file
npm install
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
