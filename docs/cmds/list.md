# ven list

List installed language versions with detailed metadata and status information.

## Overview

The `list` command displays all installed versions with rich metadata including:

- ✅ Version status (LTS, CURRENT, DEPRECATED)
- ✅ Active version indicator
- ✅ Disk space usage
- ✅ Installation dates
- ✅ JSON output for scripting
- ✅ Package compatibility estimates

## Usage

### Basic Listing

```bash
ven list [language]
```

### Examples

#### List Node.js Versions

```bash
ven list
ven list node
```

**Output:**
```
  node (3 versions installed)

    ▸ 20.11.0  [LTS] ⭐ - Active LTS (Recommended)
    • 22.3.0   [CURRENT] - Active development
    • 18.20.2  [LTS] ⭐ - Maintenance LTS

  [ACTIVE] Currently active: 20.11.0
```

#### Verbose Mode (Disk Size + Dates)

```bash
ven list --verbose
```

**Output:**
```
  node (3 versions installed)

    ▸ 20.11.0  [LTS] ⭐  45.2 MB  Installed: 2024-03-15
    • 22.3.0   [CURRENT]  48.7 MB  Installed: 2024-03-20
    • 18.20.2  [LTS] ⭐  42.1 MB  Installed: 2024-01-10

  [ACTIVE] Currently active: 20.11.0
  [DISK] Total disk space: 136.0 MB
```

#### JSON Output

```bash
ven list --json
```

**Output:**
```json
{
  "language": "node",
  "count": 3,
  "active_version": "20.11.0",
  "versions": [
    {
      "version": "20.11.0",
      "status": "LTS",
      "description": "Active LTS (Recommended)",
      "is_active": true
    },
    {
      "version": "22.3.0",
      "status": "CURRENT",
      "description": "Active development",
      "is_active": false
    },
    {
      "version": "18.20.2",
      "status": "LTS",
      "description": "Maintenance LTS",
      "is_active": false
    }
  ]
}
```

#### JSON with Full Metadata

```bash
ven list --json --verbose
```

**Output:**
```json
{
  "language": "node",
  "count": 3,
  "active_version": "20.11.0",
  "total_size_bytes": 142729830,
  "total_size_human": "136.0 MB",
  "versions": [
    {
      "version": "20.11.0",
      "status": "LTS",
      "description": "Active LTS (Recommended)",
      "is_active": true,
      "size_bytes": 47398420,
      "size_human": "45.2 MB",
      "installed_date": "2024-03-15"
    }
  ]
}
```

---

## Command Reference

### Syntax

```bash
ven list [OPTIONS] [language]
```

### Arguments

| Argument | Required | Description | Example |
|----------|----------|-------------|---------|
| `language` | No | Language to list | `node` (default) |

### Options

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--verbose` | `-v` | Show disk size and installation dates | `false` |
| `--json` | - | Output as JSON for scripting | `false` |

---

## Output Modes

### 1. Normal Mode (Default)

**Purpose**: Quick overview with status indicators

**Features:**
- Version numbers sorted newest first
- Status tags: `[LTS] ⭐`, `[CURRENT]`, `[DEPRECATED]`
- Active version highlighted with `▸` (green)
- Installed versions marked with `•` (dimmed)
- Active version notice
- Deprecated version warnings

**When to use:**
- Daily development
- Quick version checks
- Verifying installation

### 2. Verbose Mode

**Purpose**: Detailed information for maintenance

**Features:**
- Everything from normal mode
- Disk space per version
- Installation dates
- Total disk space used
- Cleanup recommendations

**When to use:**
- Disk space optimization
- Auditing installations
- Removing old versions

### 3. JSON Mode

**Purpose**: Machine-readable output for automation

**Features:**
- Structured data
- Script-friendly
- CI/CD integration
- Custom formatting

**When to use:**
- Shell scripts
- CI/CD pipelines
- Programmatic access
- Data processing

---

## Version Status Indicators

### LTS (Long Term Support) ⭐

**Versions**: 18, 20, 22 (even major numbers)

**Characteristics:**
- 30 months of active support
- 12 months of maintenance
- Highest package compatibility (~98%)
- Recommended for production

**Display:**
```
20.11.0  [LTS] ⭐ - Active LTS (Recommended)
```

### CURRENT

**Versions**: Latest odd major number (21, 23, etc.)

**Characteristics:**
- Latest features
- 6 months active support
- Good package compatibility (~95%)
- Not recommended for production

**Display:**
```
22.3.0   [CURRENT] - Active development
```

### DEPRECATED

**Versions**: End-of-life releases (16, 17, 19, etc.)

**Characteristics:**
- No security updates
- Low package compatibility (<80%)
- Should be removed

**Display:**
```
16.20.2  [DEPRECATED] - End-of-life
```

**Warning:**
```
  [TIP] 1 deprecated version(s) - consider removing to free space
```

---

## Active Version Detection

The list command automatically detects which version is **active** in the current directory by:

1. Finding `ven.toml` (walks up directory tree)
2. Reading `runtime.node` value
3. Resolving version spec (e.g., `20` → `20.11.0`)
4. Matching against installed versions

**Example:**
```toml
# ven.toml
[runtime]
node = "20"  # Will resolve to installed 20.x.x version
```

**Output:**
```
  ▸ 20.11.0  [LTS] ⭐ - Active LTS (Recommended)  ← Active version
```

---

## Disk Space Calculation

### How It Works

The verbose mode recursively calculates directory size:

```rust
fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            total_size += entry.metadata()?.len();
        } else if path.is_dir() {
            total_size += calculate_dir_size(&path)?;
        }
    }
    Ok(total_size)
}
```

### Size Formatting

Automatic human-readable formatting:

| Bytes | Display |
|-------|---------|
| < 1024 | `512 B` |
| < 1 MB | `45.2 KB` |
| < 1 GB | `136.0 MB` |
| >= 1 GB | `1.25 GB` |

---

## Installation Date Detection

Retrieves directory creation time and formats as `YYYY-MM-DD`:

```bash
# Output
20.11.0  [LTS] ⭐  45.2 MB  Installed: 2024-03-15
```

**Note**: Date calculation is approximate on some platforms due to filesystem limitations.

---

## JSON Schema

### VersionInfo Object

```json
{
  "version": "string",           // Version number
  "status": "string",            // LTS, CURRENT, DEPRECATED, UNKNOWN
  "description": "string",       // Human-readable status
  "is_active": boolean,          // Currently active in ven.toml
  "size_bytes": "number?",       // Only in --verbose mode
  "size_human": "string?",       // Only in --verbose mode
  "installed_date": "string?"    // Only in --verbose mode (YYYY-MM-DD)
}
```

### ListOutput Object

```json
{
  "language": "string",          // Language name
  "count": "number",             // Number of installed versions
  "active_version": "string?",   // Currently active version
  "total_size_bytes": "number?", // Only in --verbose mode
  "total_size_human": "string?", // Only in --verbose mode
  "versions": [VersionInfo]      // Array of version objects
}
```

---

## Use Cases

### 1. Daily Development

```bash
# Quick check
ven list

# See which version is active
ven list | grep "▸"
```

### 2. Disk Cleanup

```bash
# Find deprecated versions
ven list --verbose | grep DEPRECATED

# Check total disk usage
ven list --verbose | grep "Total disk space"
```

### 3. CI/CD Integration

```bash
# Check if version is installed
if ven list --json | jq -e '.versions[] | select(.version == "20.11.0")'; then
  echo "Node 20.11.0 is installed"
fi

# Get active version
ACTIVE=$(ven list --json | jq -r '.active_version')
echo "Active version: $ACTIVE"
```

### 4. Automation Scripts

```bash
#!/bin/bash
# Get all LTS versions
ven list --json | jq -r '.versions[] | select(.status == "LTS") | .version'

# Get total disk usage in bytes
ven list --json --verbose | jq -r '.total_size_bytes'

# Check if any deprecated versions exist
HAS_DEPRECATED=$(ven list --json | jq '[.versions[] | select(.status == "DEPRECATED")] | length')
if [ "$HAS_DEPRECATED" -gt 0 ]; then
  echo "Warning: Deprecated versions found"
fi
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/list.rs`](../../src/cli/list.rs) (397 lines)

### Key Functions

```rust
// Main entry point
cmd_list(language, verbose, json)

// Detection
detect_active_version(language)      // Find active from ven.toml

// Display modes
display_versions_with_metadata()     // Normal mode
display_verbose_mode()               // Verbose mode
output_json()                        // JSON mode

// Helpers
get_version_status(version)          // LTS/CURRENT/DEPRECATED
calculate_dir_size(path)             // Recursive size calculation
format_bytes(bytes)                  // Human-readable formatting
get_installation_date(path)          // Date formatting
```

### Dependencies

```toml
colored 2       # Terminal colors
serde 1         # JSON serialization
dirs 5          # Home directory detection
```

---

## Troubleshooting

### No Versions Installed

```bash
ven list
```

**Output:**
```
[WARN] No node versions installed. Run: ven install node latest
```

**Solution:**
```bash
ven install node lts
```

### Active Version Not Detected

**Problem**: `ven list` shows no active version

**Possible causes:**
1. No `ven.toml` in current directory
2. `ven.toml` missing `[runtime]` section
3. Specified version not installed

**Solution:**
```bash
# Check if ven.toml exists
ls ven.toml

# Verify runtime section
cat ven.toml

# Install the specified version
ven install node 20
```

### JSON Parse Error

**Problem**: Invalid JSON output

**Solution:**
```bash
# Verify output
ven list --json | jq .

# Should pretty-print valid JSON
```

---

## Related Commands

- [`ven install`](install.md) - Install new versions
- [`ven status`](status.md) - Check current project config
- [`ven setup`](setup.md) - Configure auto-switching

---

## Next Steps

### Remove Deprecated Versions

```bash
# See what's installed
ven list --verbose

# Remove old versions (manual for now)
rm -rf ~/.ven/node/16.20.2   # Unix
Remove-Item -Recurse ~/.ven/node/16.20.2  # Windows
```

### Check Active Version in Project

```bash
cd myproject/
ven list  # Shows which version is active
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
