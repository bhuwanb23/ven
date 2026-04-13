# ven upgrade

Preview and apply package upgrades with compatibility checking.

## Overview

The `upgrade` command provides **safe package upgrades** by showing a preview of changes before applying them.

Unlike `npm update` (which upgrades blindly), `ven upgrade`:

- ✅ Shows current → latest compatible version
- ✅ Verifies Node.js compatibility
- ✅ Displays release notes preview
- ✅ Requires explicit `--apply` flag
- ✅ Updates `ven.toml` automatically

## Usage

### Preview Mode (Default)

```bash
ven upgrade <package>
```

### Apply Mode

```bash
ven upgrade <package> --apply
```

### Examples

#### Preview Upgrade

```bash
ven upgrade express
```

**Output:**
```
  express 4.18.2  →  4.21.2  (latest compatible)

  Compatibility:
  ✓ Node 20.11.0 supported

  Release notes: See full changelog: npmjs.com/package/express/v/4.21.2

  Run  ven upgrade express --apply  to upgrade
```

#### Apply Upgrade

```bash
ven upgrade express --apply
```

**Output:**
```
[DOWNLOAD] Installing express@4.21.2...

added 2 packages, removed 1 package, and changed 5 packages in 3s
[OK] Installed express@4.21.2
  ✓ Updated ven.toml
```

#### Already Up to Date

```bash
ven upgrade express
```

**Output:**
```
✓ express is already up to date (4.21.2)
```

---

## Command Reference

### Syntax

```bash
ven upgrade [OPTIONS] <package>
```

### Arguments

| Argument | Required | Description | Example |
|----------|----------|-------------|---------|
| `package` | Yes | Package name to upgrade | `express`, `react` |

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--apply` | Actually perform the upgrade | `false` |

---

## How It Works

### Upgrade Process

```
1. Read current Node version from ven.toml
   ↓
2. Get currently installed version from node_modules
   ↓
3. Fetch latest compatible version from npm
   ↓
4. Compare versions
   ↓
5. If same: show "up to date" message
   ↓
6. If different: show preview
   ↓
7. If --apply: install new version
   ↓
8. Update ven.toml
```

### Example Walkthrough

**Current State:**
```json
// node_modules/express/package.json
{
  "name": "express",
  "version": "4.18.2"
}
```

**Command:**
```bash
ven upgrade express
```

**Process:**
1. Read Node version: `20.11.0` from ven.toml
2. Read current: `4.18.2` from node_modules
3. Fetch npm registry: latest compatible is `4.21.2`
4. Compare: `4.18.2` ≠ `4.21.2`
5. Show preview

---

## Preview Mode (Default)

### Purpose

Show what **would** happen without making changes.

### Output Format

```
  <package> <current>  →  <latest>  (latest compatible)

  Compatibility:
  ✓ Node <version> supported

  Release notes: <changelog_url>

  Run  ven upgrade <package> --apply  to upgrade
```

### Real Example

```bash
ven upgrade typescript
```

**Output:**
```
  typescript 5.2.2  →  5.3.3  (latest compatible)

  Compatibility:
  ✓ Node 20.11.0 supported

  Release notes: See full changelog: npmjs.com/package/typescript/v/5.3.3

  Run  ven upgrade typescript --apply  to upgrade
```

### Benefits

- **No surprises**: See exact version before installing
- **Compatibility assured**: Verified against your Node version
- **Informed decisions**: Check changelog before upgrading
- **Reversible**: Nothing changes until `--apply`

---

## Apply Mode

### Purpose

Actually perform the upgrade.

### Process

```bash
ven upgrade express --apply
```

**What happens:**
1. Run `npm install express@4.21.2`
2. npm updates node_modules
3. npm updates package-lock.json
4. ven updates ven.toml

### Output

```
[DOWNLOAD] Installing express@4.21.2...

added 2 packages, removed 1 package, and changed 5 packages in 3s
[OK] Installed express@4.21.2
  ✓ Updated ven.toml
```

---

## Version Resolution

### Compatibility Check

ven ensures the new version works with your Node.js version:

```rust
let info = fetch_npm_info(package)?;
let latest = find_compatible_version(&info, &node_version)
    .ok_or_else(|| anyhow::anyhow!("No compatible version found"))?;
```

**Example:**

**Node 18.15.0:**
```bash
ven upgrade next
```

**Result:**
```
  next 13.4.0  →  13.5.6  (latest compatible)
```

Not `14.x.x` because Next.js 14 requires Node >= 18.17.0.

---

## Release Notes

### Current Implementation (Phase 1)

Shows link to npm changelog:

```
Release notes: See full changelog: npmjs.com/package/express/v/4.21.2
```

### Future Enhancement (Phase 2)

Will fetch and display actual changelog:

```
Release notes (4.18.2 → 4.21.2):
• Fixed response header handling
• Improved error messages
• Security patches for XSS vulnerability
• Updated dependencies
```

---

## Use Cases

### 1. Regular Maintenance

```bash
# Check for updates
ven upgrade express

# If looks good, apply
ven upgrade express --apply
```

### 2. Security Patches

```bash
# Check if security fix available
ven upgrade lodash

# See version bump
# lodash 4.17.20  →  4.17.21

# Apply immediately
ven upgrade lodash --apply
```

### 3. Major Version Decisions

```bash
# Check major upgrade
ven upgrade react

# See: react 17.0.2  →  18.2.0

# Read changelog first
# Open: npmjs.com/package/react/v/18.2.0

# Decide to upgrade or wait
ven upgrade react --apply
```

### 4. Batch Upgrades

```bash
# Upgrade all packages (manual loop)
for pkg in express typescript eslint prettier; do
  ven upgrade $pkg --apply
done
```

---

## Comparison with npm

### npm update

```bash
npm update express
```

**Behavior:**
- Upgrades within semver range
- No compatibility check
- No preview
- May break things

### ven upgrade

```bash
ven upgrade express
ven upgrade express --apply
```

**Behavior:**
- Shows preview first
- Checks Node compatibility
- Finds highest compatible version
- Requires explicit confirmation
- Safe and informed

---

## Error Handling

### No Compatible Version

```bash
ven upgrade brand-new-package
```

**Output:**
```
Error: No compatible version found
```

**Solution:**
```bash
# Upgrade Node.js first
ven install node 20

# Try again
ven upgrade brand-new-package
```

### Package Not Installed

```bash
ven upgrade express
```

**Output:**
```
  express unknown  →  4.21.2  (latest compatible)
  
  Compatibility:
  ✓ Node 20.11.0 supported
  
  Release notes: See full changelog: npmjs.com/package/express/v/4.21.2
  
  Run  ven upgrade express --apply  to upgrade
```

**Note:** Shows `unknown` as current version (not installed yet).

### No ven.toml

```bash
cd /tmp/
ven upgrade express
```

**Output:**
```
Error: No ven.toml found. Run: ven init
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/upgrade.rs`](../../src/cli/upgrade.rs) (43 lines)

### Code

```rust
pub fn cmd_upgrade(package: &str, apply: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    // Get currently installed version
    let current_ver = get_installed_version(package)
        .unwrap_or_else(|_| "unknown".to_string());

    // Fetch latest compatible version
    let info = fetch_npm_info(package)?;
    let latest = find_compatible_version(&info, &node_version)
        .ok_or_else(|| anyhow::anyhow!("No compatible version found"))?;

    if current_ver == latest {
        println!("{} {} is already up to date ({})", 
            "✓".green(), package.bold(), latest);
        return Ok(());
    }

    println!("\n  {} {}  →  {}  (latest compatible)", 
        package.bold(), current_ver.dimmed(), latest.green());
    println!("\n  Compatibility: {} Node {} supported", 
        "✓".green(), node_version);

    let notes = fetch_release_notes(package, &current_ver, &latest);
    println!("\n  Release notes: {}", notes.dimmed());

    if !apply {
        println!("\n  Run  {} to upgrade", 
            format!("ven upgrade {} --apply", package).bold());
        return Ok(());
    }

    npm_install(package, &latest)?;
    update_ven_toml_package(package, &latest)?;
    Ok(())
}
```

### Dependencies

```rust
use crate::core::{load_config, packages::*};
use crate::cli::add::update_ven_toml_package;
use colored::Colorize;
```

---

## Best Practices

### 1. Always Preview First

```bash
# Never jump straight to --apply
ven upgrade package

# Review the changes
# Check changelog
# Then decide
ven upgrade package --apply
```

### 2. Upgrade in Stages

```bash
# Minor/patch upgrades (usually safe)
ven upgrade express --apply

# Major upgrades (review carefully)
ven upgrade react  # See 17 → 18
# Read changelog
# Test in dev environment
ven upgrade react --apply
```

### 3. Test After Upgrading

```bash
# Upgrade
ven upgrade jest --apply

# Run tests
npm test

# If tests fail, downgrade
ven install jest@previous-version
```

### 4. Use with CI/CD

```bash
#!/bin/bash
# Automated upgrade script

PACKAGES="express typescript eslint prettier"

for pkg in $PACKAGES; do
  echo "Upgrading $pkg..."
  ven upgrade $pkg --apply
done

# Run tests
npm test

# Commit changes
git add package.json package-lock.json ven.toml
git commit -m "chore: upgrade dependencies"
```

---

## Troubleshooting

### Upgrade Doesn't Change Version

**Problem**: `ven upgrade` shows same version

**Possible causes:**
1. Already on latest compatible version
2. Node version constraint prevents upgrade
3. Package has no newer compatible releases

**Solution:**
```bash
# Check Node version
ven status

# If outdated, upgrade Node
ven install node 20

# Try package upgrade again
ven upgrade package --apply
```

### Breaking Changes After Upgrade

**Problem**: Package upgrade breaks app

**Solution:**
```bash
# Check what changed
git diff package.json

# Downgrade to previous version
ven install package@previous-version

# Or pin version in ven.toml
# package = "4.18.2"  # Previous version
```

### Compatibility Check Fails

**Problem**: "No compatible version found"

**Solution:**
```bash
# Check Node version constraint
ven status

# Upgrade Node if needed
ven install node 20

# Retry upgrade
ven upgrade package --apply
```

---

## Related Commands

- [`ven add`](add.md) - Add new packages
- [`ven remove`](remove.md) - Remove packages safely
- [`ven status`](status.md) - Check project config

---

## Next Steps

After upgrading:

```bash
# Verify everything works
npm test

# Check status
ven status

# Commit changes
git add .
git commit -m "chore: upgrade packages"
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
