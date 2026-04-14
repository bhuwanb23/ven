# ven status

Show the current project's version and package configuration.

## Overview

The `status` command provides a quick overview of the active project configuration by reading the nearest `ven.toml` file.

## Usage

```bash
ven status
```

### Examples

#### With ven.toml

```bash
cd myproject/
ven status
```

**Output:**
```
  ven status /home/user/myproject
  node 20.11.0
  packages 5 packages declared
```

#### Without ven.toml

```bash
cd /tmp/
ven status
```

**Output:**
```
  ven status /tmp
  No ven.toml found in this directory tree.
  Run: ven init   to create one.
```

---

## What It Shows

### 1. Project Path

Displays the current working directory path.

### 2. Active Runtime Version

Shows the Node.js version specified in `ven.toml`:

```toml
[runtime]
node = "20.11.0"  # ← This value
```

### 3. Package Count

Counts the number of packages declared in the `[packages]` section:

```toml
[packages]
express = "^4.18.2"
typescript = "^5.3.0"
dotenv = "^16.3.1"
# 3 packages declared
```

---

## Command Reference

### Syntax

```bash
ven status
```

### Arguments

None - uses current working directory.

### Options

None - simple command with no flags.

---

## How It Works

### Configuration Discovery

1. **Start**: Current directory (`$PWD`)
2. **Search**: Walk up directory tree looking for `ven.toml`
3. **Parse**: Read and parse the TOML file
4. **Display**: Show runtime version and package count

### Example Walk

```bash
# Directory structure
/home/user/projects/app/src/components/
# ven.toml is at /home/user/projects/app/ven.toml

cd /home/user/projects/app/src/components/
ven status
```

**Process:**
```
src/components/  → No ven.toml
src/             → No ven.toml
app/             → Found ven.toml! ✅
```

**Output:**
```
  ven status /home/user/projects/app/src/components
  node 20.11.0
  packages 8 packages declared
```

---

## Use Cases

### 1. Quick Project Check

```bash
# Verify you're in the right project
ven status

# Shows which Node version this project uses
```

### 2. Monorepo Navigation

```bash
cd monorepo/frontend/
ven status
# node 20.11.0

cd ../backend/
ven status
# node 22.3.0
```

### 3. Before Installing Packages

```bash
# Check current setup
ven status

# Install with confidence
ven add express  # Will check compatibility with shown Node version
```

### 4. Team Onboarding

```bash
# New developer joins
git clone <repo>
cd project/
ven status  # See what versions are needed

# Install required version
ven install node 20
```

---

## Output Format

### With Configuration

```
  ven status <path>
  node <version>
  packages <count> packages declared
```

### Without Configuration

```
  ven status <path>
  No ven.toml found in this directory tree.
  Run: ven init   to create one.
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/status.rs`](../../src/cli/status.rs) (29 lines)

### Code

```rust
pub fn cmd_status() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config = load_config(&cwd)?;

    println!("\n  {} {}", "ven status".bold(), cwd.display());

    match config {
        None => {
            println!("  No ven.toml found in this directory tree.");
            println!("  Run: ven init   to create one.");
        }
        Some(cfg) => {
            let node_ver = &cfg.runtime.node;
            println!("  {} {}", "node".bold(), node_ver.green());
            if !cfg.packages.is_empty() {
                println!("  {} {} packages declared", "packages".bold(), cfg.packages.len());
            }
        }
    }
    println!();
    Ok(())
}
```

### Dependencies

```rust
use crate::core::load_config;  // Finds and parses ven.toml
use colored::Colorize;         // Terminal colors
```

---

## ven.toml Structure

### Minimal

```toml
[runtime]
node = "20.11.0"
```

**Output:**
```
  node 20.11.0
```

### With Packages

```toml
[runtime]
node = "20.11.0"

[packages]
express = "^4.18.2"
react = "^18.2.0"
```

**Output:**
```
  node 20.11.0
  packages 2 packages declared
```

### Full Example

```toml
[runtime]
node = "20"

[packages]
express = "^4.18.2"
typescript = "^5.3.0"
dotenv = "^16.3.1"
cors = "^2.8.5"
morgan = "^1.10.0"

[env]
NODE_ENV = "development"
PORT = "3000"
```

**Output:**
```
  node 20
  packages 5 packages declared
```

---

## Troubleshooting

### Wrong Directory

**Problem**: `ven status` shows different project

**Solution:**
```bash
# Check where you are
pwd

# Navigate to correct directory
cd /path/to/project/
ven status
```

### ven.toml Not Found

**Problem**: "No ven.toml found" but file exists

**Possible causes:**
1. File is named incorrectly (e.g., `Ven.toml`)
2. File is in wrong location
3. File permissions issue

**Solution:**
```bash
# Check file exists
ls -la ven.toml

# Verify content
cat ven.toml

# Create if missing
ven init
```

### Empty Runtime Section

**Problem**: ven.toml exists but no node version shown

**Check:**
```toml
[runtime]
# node = "..."  ← Missing!
```

**Fix:**
```toml
[runtime]
node = "20.11.0"
```

---

## Related Commands

- [`ven init`](init.md) - Create ven.toml
- [`ven list`](list.md) - View installed versions
- [`ven install`](install.md) - Install Node.js versions

---

## Next Steps

After checking status:

```bash
# If no ven.toml
ven init

# If version not installed
ven install node 20

# If adding packages
ven add express
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
