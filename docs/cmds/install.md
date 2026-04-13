# ven install

Install language versions with native download directly from official sources.

## Overview

The `install` command downloads and installs language runtimes (Node.js, Python, etc.) **without requiring external tools** like fnm, pyenv, or nvm. ven handles everything natively:

- ✅ Direct download from official sources
- ✅ SHA256 checksum verification
- ✅ Progress bars with download speed
- ✅ Archive caching (re-downloads avoided)
- ✅ Post-install validation
- ✅ Cross-platform support (Windows/macOS/Linux)

## Usage

### Basic Installation

```bash
ven install <language> <version>
```

### Examples

#### Install Specific Version

```bash
ven install node 20.11.0
```

**Output:**
```
[FETCH] Fetching node release list...
[OK] Resolved to node 20.11.0
[ARROW] Preparing to download Node 20.11.0...
• URL: https://nodejs.org/dist/v20.11.0/node-v20.11.0-win-x64.zip
[ARROW] Downloading...
⠙ [00:15] [████████████████████████████████████] 32.5 MB/32.5 MB (15s)
Download complete
• Verifying checksum...
[OK] Checksum verified
[ARROW] Extracting to C:\Users\you\.ven\node\20.11.0...
[OK] Extraction complete
[OK] Node 20.11.0 installed successfully
• Binary: C:\Users\you\.ven\node\20.11.0\node.exe

[CHECK] Validating installation...
  [OK] Binary: C:\Users\you\.ven\node\20.11.0\node.exe
  [OK] Version: node 20.11.0
  [OK] PATH: Ready to use

[SUCCESS] node 20.11.0 installed successfully!
  [TIP] Run: ven init   to create a project
```

#### Install Latest Patch Version

```bash
ven install node 20
```

Resolves `20` to the latest `20.x.x` available (e.g., `20.20.2`).

**Output:**
```
[RESOLVE] Resolving node 20 to latest patch version...
[OK] Resolved to node 20.20.2
[DOWNLOAD] Installing Node 20.20.2...
...
```

#### Install LTS Version

```bash
ven install node lts
```

Fetches and installs the latest Long Term Support version.

**Output:**
```
[FETCH] Fetching node release list...
[OK] Resolved to node 20.20.2
[DOWNLOAD] Installing Node 20.20.2...
...
```

#### Install Latest Stable

```bash
ven install node latest
```

Installs the most recent stable release (may be a non-LTS version).

---

### Interactive Mode

#### Show Available Versions

```bash
ven install node
```

Displays a list of available versions with metadata and lets you select one.

**Output:**
```
[PKG] Available node Versions

  [SPECIAL] Quick Options:
    latest  - Install latest stable release
    lts     - Install latest LTS version (Recommended)

  [VERSIONS] Latest Available Versions:
     1.          22.22.2  [CURRENT]  (~95% pkg compat)
     2.          22.20.0  [CURRENT]  (~95% pkg compat)
     3.          20.20.2  [LTS] ⭐ (~98% pkg compat) [Recommended]
     4.          20.18.0  [LTS] ⭐ (~98% pkg compat) [Recommended]
     5.          18.20.2  [LTS] ⭐ (~95% pkg compat) [Maintenance]
     ...

  [INFO] ... and 150 more versions (use major version like 20, 22, 18)

[TIP] Recommended: node 20 (LTS - Best compatibility)

? Select version (use arrow keys):
  ▸ latest - Latest stable release
    lts    - Latest LTS version (Recommended)
    --- Press ENTER to select ---
     1. 22.22.2 (CURRENT)
     2. 22.20.0 (CURRENT)
     3. 20.20.2 (LTS)
     ...
```

#### Full Interactive Wizard

```bash
ven install
```

Guides you through:
1. Language selection (node, python, etc.)
2. Version selection with metadata
3. Installation with progress tracking
4. Post-install validation

---

## Command Reference

### Syntax

```bash
ven install [OPTIONS] <language> [version]
```

### Arguments

| Argument | Required | Description | Example |
|----------|----------|-------------|---------|
| `language` | No | Language to install | `node`, `python` |
| `version` | No | Version to install | `20.11.0`, `20`, `lts`, `latest` |

### Behavior Matrix

| Command | Behavior |
|---------|----------|
| `ven install node 20.11.0` | Install exact version |
| `ven install node 20` | Resolve `20` → latest `20.x.x` |
| `ven install node lts` | Install latest LTS |
| `ven install node latest` | Install latest stable |
| `ven install node` | Show version list → interactive select |
| `ven install` | Full wizard (language + version) |
| `ven install python 3.12` | (Future) Install Python |

---

## Version Resolution

### Supported Formats

| Format | Example | Resolution |
|--------|---------|------------|
| Exact version | `20.11.0` | Used as-is |
| Major only | `20` | Latest `20.x.x` from nodejs.org |
| LTS alias | `lts` | Latest LTS (even major: 18, 20, 22) |
| Latest alias | `latest` | Most recent stable release |

### Resolution Process

1. **Check if exact**: If version contains `.`, use as-is
2. **Major-only**: If just a number (e.g., `20`), fetch from nodejs.org and find highest patch
3. **Alias**: If `lts` or `latest`, fetch release list and filter accordingly

---

## Storage Layout

### Download Location

```
~/.ven/                          # Root storage (VEN_STORAGE_PATH)
├── .cache/                      # Cached archives
│   ├── node-v20.11.0-win-x64.zip
│   └── node-v22.3.0-win-x64.zip
└── node/                        # Installed versions
    ├── 20.11.0/                 # Windows: binaries in root
    │   ├── node.exe
    │   ├── npm.cmd
    │   └── ...
    └── 22.3.0/
        ├── node.exe
        └── ...
```

**Unix Layout:**
```
~/.ven/node/20.11.0/
└── bin/                         # Unix: binaries in bin/
    ├── node
    ├── npm
    └── npx
```

### Custom Storage Path

```bash
# Override default location
export VEN_STORAGE_PATH="/opt/ven"  # Unix
$env:VEN_STORAGE_PATH = "D:\ven"    # Windows (PowerShell)
```

---

## Features

### 1. Native Download (No External Tools)

**Old approach**: Required `fnm`, `nvm`, or similar
**New approach**: ven downloads directly from nodejs.org

**Benefits:**
- Zero external dependencies
- Faster installation (no wrapper overhead)
- Full control over download process
- Better error messages

### 2. SHA256 Checksum Verification

Every download is verified against the official `SHASUMS256.txt` file from nodejs.org.

**Process:**
1. Download archive
2. Fetch checksum from nodejs.org
3. Calculate SHA256 of downloaded file
4. Compare checksums
5. If mismatch: delete corrupted file and abort

**Output:**
```
• Verifying checksum...
[OK] Checksum verified
```

**On failure:**
```
• Verifying checksum...
[ERROR] Checksum mismatch! Corrupted download removed. Try again.
```

### 3. Archive Caching

Downloaded archives are cached in `~/.ven/.cache/` to avoid re-downloading.

**Behavior:**
```bash
ven install node 20.11.0   # Downloads and caches
ven install node 20.11.0   # Uses cached archive (instant)
```

**Output when cached:**
```
[OK] Using cached archive
• Verifying checksum...
[OK] Checksum verified
```

### 4. Progress Bars

Real-time download progress with speed and ETA:

```
⠙ [00:15] [████████████████████████████████████] 32.5 MB/32.5 MB (15s)
```

**Features:**
- Animated spinner
- Elapsed time
- Progress bar with percentage
- Download speed
- Estimated time remaining
- Total size

### 5. Post-Install Validation

After installation, ven validates:
1. **Binary exists**: Checks if `node.exe` (Windows) or `node` (Unix) exists
2. **Version matches**: Verifies installed version matches requested
3. **PATH ready**: Confirms binary directory is accessible

**Output:**
```
[CHECK] Validating installation...
  [OK] Binary: C:\Users\you\.ven\node\20.11.0\node.exe
  [OK] Version: node 20.11.0
  [OK] PATH: Ready to use

[SUCCESS] node 20.11.0 installed successfully!
  [TIP] Run: ven init   to create a project
```

### 6. Smart Error Messages

Provides actionable suggestions when things go wrong.

**Deprecated version:**
```
[ERROR] Node.js 16 is not available or deprecated

[INFO] Available LTS versions:
  - 18.20.2 (Maintenance LTS)
  - 20.20.2 (Active LTS) <- Recommended
  - 22.22.2 (Current)

[TIP] Try: ven install node 20
```

**Future version:**
```
[ERROR] Node.js 25 is not available yet

[INFO] Latest available versions:
  - 22.22.2 (Current)
  - 20.20.2 (Active LTS)

[TIP] Try: ven install node 22
```

---

## Platform Support

### Windows

- **Archive format**: `.zip`
- **Binary location**: `~/.ven/node/VERSION/node.exe`
- **Shell**: PowerShell
- **Detection**: Automatic via `PSModulePath` env var

### macOS

- **Archive format**: `.tar.gz`
- **Binary location**: `~/.ven/node/VERSION/bin/node`
- **Shell**: bash/zsh
- **Architectures**: x64 (Intel), arm64 (Apple Silicon)

### Linux

- **Archive format**: `.tar.gz`
- **Binary location**: `~/.ven/node/VERSION/bin/node`
- **Shell**: bash/fish
- **Architectures**: x64, arm64

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/install.rs`](../../src/cli/install.rs) (391 lines)
- **Downloader**: [`src/core/download.rs`](../../src/core/download.rs) (284 lines)
- **Extractor**: [`src/core/extract.rs`](../../src/core/extract.rs) (165 lines)
- **Node plugin**: [`src/plugins/node.rs`](../../src/plugins/node.rs) (55 lines)

### Key Functions

```rust
// CLI layer
cmd_install(language, version)              // Direct install
cmd_install_interactive()                   // Full wizard
cmd_install_with_version_list(language)     // Show versions, then install

// Resolution
resolve_major_version(plugin, major)        // 20 → 20.20.2

// Core
NodeDownloader::download(version)           // Download with progress
NodeDownloader::verify_checksum()           // SHA256 verification
install_node_native(downloader, version)    // Full install pipeline
```

### Dependencies

```toml
reqwest 0.12          # HTTP client (blocking mode)
indicatif 0.17        # Progress bars
sha2 0.10             # SHA256 checksums
zip 0.6               # ZIP extraction (Windows)
flate2 1.0            # GZIP decompression
tar 0.4               # TAR extraction (Unix)
colored 2             # Terminal colors
dialoguer 0.11        # Interactive prompts
```

---

## Troubleshooting

### Download Fails

**Problem**: Network error or timeout
```
[ERROR] Cannot reach nodejs.org: connection timed out
```

**Solution**:
1. Check internet connection
2. Try again (caching will help on retry)
3. Set proxy if needed (via `HTTP_PROXY` env var)

### Checksum Mismatch

**Problem**: Corrupted download
```
[ERROR] Checksum mismatch! Corrupted download removed. Try again.
```

**Solution**:
1. Run command again (will re-download)
2. If persists, check disk health
3. Verify no antivirus interference

### Binary Not Found

**Problem**: Extraction failed
```
[ERROR] Installation verification failed: binary not found
```

**Solution**:
1. Check `~/.ven/node/VERSION/` directory exists
2. Verify extraction completed (no disk space issues)
3. Try reinstalling: `ven install node <version>`

### Permission Denied (Unix)

**Problem**: Cannot write to `~/.ven/`
```
Error: Permission denied (os error 13)
```

**Solution**:
```bash
# Fix ownership
sudo chown -R $USER:$USER ~/.ven/

# Or use custom location
export VEN_STORAGE_PATH="$HOME/.local/ven"
```

---

## Related Commands

- [`ven list`](list.md) - View installed versions
- [`ven setup`](setup.md) - Configure shell auto-switching
- [`ven status`](status.md) - Check current project config
- [`ven init`](init.md) - Create new project

---

## Next Steps

After installing Node.js:

```bash
# 1. Set up auto-switching
ven setup

# 2. Create a project
ven init --template

# 3. Install packages
ven add express
ven add typescript
```

For a complete workflow guide, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
