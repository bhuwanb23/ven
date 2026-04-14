# ven shell (Internal)

Shell integration commands for auto-switching (called by eval hook).

## ⚠️ Internal Command

**Note:** This command is **not meant for direct user interaction**. It's called automatically by the shell hook installed via [`ven setup`](setup.md).

## Overview

The `shell` command provides low-level shell integration functionality:

- Generate hook code for different shells
- Compute PATH exports for directories
- Support bash, zsh, fish, and PowerShell

## Usage

```bash
ven shell <subcommand> [arguments]
```

This command is **hidden** from `ven --help`.

---

## Subcommands

### 1. `ven shell hook <shell>`

Generate shell hook code for the specified shell.

#### Supported Shells

| Shell | Platform | Hook Type |
|-------|----------|-----------|
| `bash` | Linux/macOS | cd override |
| `zsh` | macOS | cd override |
| `fish` | Linux/macOS | PWD variable trigger |
| `powershell` / `pwsh` | Windows | cd alias |

#### Example

```bash
ven shell hook bash
```

**Output:**
```bash
# ven shell hook
__ven_activate() {
    local exports
    exports=$(ven shell activate "$PWD" 2>/dev/null)
    if [ -n "$exports" ]; then
        eval "$exports"
    fi
}
cd() { builtin cd "$@" && __ven_activate; }
__ven_activate  # activate for current directory on shell start
```

#### Usage in Setup

This is what `ven setup` does internally:

```bash
# Detect shell
shell=$(detect_shell)

# Generate hook
hook_code=$(ven shell hook $shell)

# Append to shell config
echo "$hook_code" >> ~/.bashrc
```

---

### 2. `ven shell activate <directory>`

Compute and print PATH exports for a specific directory.

#### How It Works

1. Search for `ven.toml` starting from `<directory>`
2. Walk up directory tree until found
3. Parse `ven.toml` and read `runtime.node`
4. Resolve version alias to concrete version
5. Calculate binary path for that version
6. Output shell export commands

#### Example

```bash
ven shell activate /home/user/projects/api
```

**Output (if ven.toml exists):**
```bash
export PATH="/home/user/.ven/node/20.11.0/bin:$PATH"
export VEN_NODE_VERSION="20.11.0"
export VEN_TOML="/home/user/projects/api/ven.toml"
export NODE_ENV="development"
export PORT="3000"
```

**Output (if no ven.toml):**
```
(nothing - empty output)
```

#### Platform-Specific Output

**PowerShell (Windows):**
```powershell
$env:PATH = "C:\Users\you\.ven\node\20.11.0;" + $env:PATH
$env:VEN_NODE_VERSION = "20.11.0"
$env:VEN_TOML = "C:\Users\you\projects\api\ven.toml"
$env:NODE_ENV = "development"
```

**bash/zsh (Unix):**
```bash
export PATH="/home/you/.ven/node/20.11.0/bin:$PATH"
export VEN_NODE_VERSION="20.11.0"
export VEN_TOML="/home/you/projects/api/ven.toml"
export NODE_ENV="development"
```

**fish (Unix):**
```fish
set -gx PATH "/home/you/.ven/node/20.11.0/bin" $PATH
set -gx VEN_NODE_VERSION "20.11.0"
set -gx VEN_TOML "/home/you/projects/api/ven.toml"
set -gx NODE_ENV "development"
```

---

## Command Reference

### Syntax

```bash
ven shell hook <shell>
ven shell activate <directory>
```

### Subcommands

| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `hook` | `<shell>` | Generate hook code for shell |
| `activate` | `<directory>` | Compute PATH exports for directory |

---

## How Auto-Switching Works

### Complete Flow

```
User runs: cd /path/to/project/
    ↓
Shell hook intercepts cd
    ↓
Calls: ven shell activate /path/to/project/
    ↓
ven searches for ven.toml (walks up)
    ↓
Found: /path/to/project/ven.toml
    ↓
Reads: runtime.node = "20"
    ↓
Resolves: "20" → "20.11.0" (installed version)
    ↓
Calculates: ~/.ven/node/20.11.0/bin
    ↓
Outputs: export PATH="~/.ven/node/20.11.0/bin:$PATH"
    ↓
Shell evaluates: eval "$exports"
    ↓
Result: node --version shows v20.11.0
```

---

## Environment Variables

### Output Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `PATH` | Prepend Node binary directory | `~/.ven/node/20.11.0/bin:$PATH` |
| `VEN_NODE_VERSION` | Track active Node version | `20.11.0` |
| `VEN_TOML` | Track active config file | `/path/to/ven.toml` |

### From ven.toml [env] Section

Any variables in the `[env]` section are also exported:

```toml
[env]
NODE_ENV = "production"
API_URL = "https://api.example.com"
```

**Becomes:**
```bash
export NODE_ENV="production"
export API_URL="https://api.example.com"
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/shell.rs`](../../src/cli/shell.rs) (21 lines)
- **Shell logic**: [`src/shell/mod.rs`](../../src/shell/mod.rs) (153 lines)

### Key Functions

```rust
// CLI layer
cmd_shell_hook(shell)        // Print hook code
cmd_shell_activate(dir)      // Print exports

// Shell module
detect_shell()               // Auto-detect running shell
generate_hook(shell)         // Generate shell-specific hook code
compute_exports(dir)         // Calculate PATH exports for directory
```

### compute_exports Logic

```rust
pub fn compute_exports(dir: &Path) -> Result<Option<String>> {
    // Find nearest ven.toml (walks up from dir)
    let toml_path = match find_ven_toml(dir) {
        Some(p) => p,
        None    => return Ok(None), // no ven.toml — print nothing
    };

    let config = parse_ven_toml(&toml_path)?;
    let node_spec = &config.runtime.node;

    // Resolve alias ("lts", "20") to installed concrete version ("20.11.0")
    let plugin = NodePlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = resolve_node_version(node_spec, &installed)?;

    // Get the bin/ path for this resolved version
    let bin_path = plugin.bin_path(&resolved)?;

    // Generate export commands (platform-specific)
    let exports = if cfg!(target_os = "windows") {
        // PowerShell syntax
        format!("$env:PATH = \"{bin};\" + $env:PATH\n...")
    } else {
        // bash/zsh syntax
        format!("export PATH=\"{bin}:$PATH\"\n...")
    };

    Ok(Some(exports))
}
```

---

## Use Cases

### 1. Manual Activation (Without Hook)

```bash
# Activate for current directory
eval "$(ven shell activate $PWD)"

# Check what was activated
echo $VEN_NODE_VERSION
```

### 2. Custom Shell Integration

```bash
# Add to .bashrc with custom behavior
__custom_ven_activate() {
    exports=$(ven shell activate "$PWD" 2>/dev/null)
    if [ -n "$exports" ]; then
        eval "$exports"
        echo "Activated Node $VEN_NODE_VERSION"
    fi
}
cd() { builtin cd "$@" && __custom_ven_activate; }
```

### 3. Debug Auto-Switching

```bash
# See what ven would export
ven shell activate /path/to/project/

# Check if ven.toml found
# If output is empty, no ven.toml found
```

### 4. Script Integration

```bash
#!/bin/bash
# CI script that activates correct Node version

PROJECT_DIR="/path/to/project"
eval "$(ven shell activate $PROJECT_DIR)"

echo "Using Node $VEN_NODE_VERSION"
node --version
npm install
npm test
```

---

## Hook Code Examples

### bash/zsh

```bash
# ven shell hook
__ven_activate() {
    local exports
    exports=$(ven shell activate "$PWD" 2>/dev/null)
    if [ -n "$exports" ]; then
        eval "$exports"
    fi
}
cd() { builtin cd "$@" && __ven_activate; }
__ven_activate  # activate for current directory on shell start
```

**How it works:**
- `__ven_activate()`: Function that gets exports and evals them
- `cd()`: Override built-in cd to call activate after
- `builtin cd`: Call original cd command
- `&& __ven_activate`: Run activate after successful cd
- Final `__ven_activate`: Activate on shell start

---

### fish

```fish
# ven shell hook (fish)
function __ven_activate --on-variable PWD
    set exports (ven shell activate "$PWD" 2>/dev/null)
    if test -n "$exports"
        eval $exports
    end
end
__ven_activate  # activate on shell start
```

**How it works:**
- `--on-variable PWD`: Trigger when PWD changes
- No cd override needed (fish handles it)
- `set exports`: Capture output
- `eval $exports`: Apply exports

---

### PowerShell

```powershell
# ven shell hook (PowerShell)
function Set-VenLocation {
    param([string]$Path = "")
    if ($Path) {
        Set-Location $Path
    }
    $exports = ven shell activate "$PWD" 2>$null
    if ($exports) {
        Invoke-Expression $exports
    }
}
Set-Alias -Name cd -Value Set-VenLocation -Force -Option AllScope
# Activate for current directory on shell start
$_ven_exports = ven shell activate "$PWD" 2>$null
if ($_ven_exports) { Invoke-Expression $_ven_exports }
```

**How it works:**
- `Set-VenLocation`: Custom cd function
- `Set-Alias`: Override cd command
- `Invoke-Expression`: Evaluate exports
- `$env:PATH = "..." + $env:PATH`: Prepend to PATH
- `2>$null`: Suppress errors
- Final activation: Run on shell start

---

## Testing

### Test Hook Generation

```bash
# bash
ven shell hook bash

# zsh
ven shell hook zsh

# fish
ven shell hook fish

# PowerShell
ven shell hook powershell
```

### Test Activation

```bash
# In directory with ven.toml
ven shell activate $PWD

# In directory without ven.toml
cd /tmp/
ven shell activate $PWD  # Should output nothing
```

### Test Full Flow

```bash
# Create test project
mkdir /tmp/test-ven && cd /tmp/test-ven
ven init --node 20
ven install node 20

# Test activation
eval "$(ven shell activate $PWD)"

# Verify
node --version  # Should show 20.x.x
echo $VEN_NODE_VERSION  # Should show 20.x.x
```

---

## Troubleshooting

### Empty Output from activate

**Problem:** `ven shell activate` returns nothing

**Causes:**
1. No ven.toml in directory tree
2. ven.toml missing `[runtime]` section
3. Specified Node version not installed

**Solutions:**
```bash
# Check if ven.toml exists
find . -name ven.toml

# Verify content
cat ven.toml

# Check if version installed
ven list

# Install missing version
ven install node 20
```

### Hook Not Executing

**Problem:** cd doesn't trigger activation

**Solutions:**
```bash
# Check if hook is installed
grep "ven shell hook" ~/.bashrc

# Test hook code manually
ven shell activate $PWD

# Reinstall hook
ven setup
source ~/.bashrc
```

### Wrong Version Activated

**Problem:** activate returns wrong Node version

**Solutions:**
```bash
# Check ven.toml
cat ven.toml

# Check what versions are installed
ven list

# Check resolution
# If ven.toml has: node = "20"
# And installed: 20.11.0, 20.18.0
# Will resolve to: 20.18.0 (highest 20.x.x)
```

---

## Related Commands

- [`ven setup`](setup.md) - Install shell hook (user-facing)
- [`ven status`](status.md) - Check current configuration
- [`ven list`](list.md) - View installed versions

---

## Architecture Note

This command is intentionally **hidden** from users because:

1. **Internal implementation detail**: Users shouldn't need to call it directly
2. **Automated by setup**: `ven setup` handles all configuration
3. **Called by shell hook**: Automatic on every `cd`
4. **Eval-based**: Requires careful handling (security)

**Users should only interact with:**
- `ven setup` - One-time configuration
- `ven init` - Project setup
- `ven install` - Version management
- `ven add/remove/upgrade` - Package management

The shell integration happens automatically after setup.

---

## Next Steps

For user-facing documentation, see:
- [`ven setup`](setup.md) - Configure auto-switching
- [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md) - Full workflow
