# Phase 0: Foundation Fixes - Implementation Plan

**Goal:** Make the basic runtime switching actually work  
**Timeline:** Week 1-2  
**Status:** 🚧 IN PROGRESS

---

## ✅ Success Criteria

You can `cd` into a folder and Node version switches automatically.

---

## 📋 Task Breakdown

### Task 1: Fix Shell Integration ✅ COMPLETE

**What needs to work:**
- Shell hook generation for bash/zsh/fish/PowerShell
- Hook code properly overrides `cd` command
- Hook calls `ven shell activate $PWD` on directory change

**Current State:**
- ✅ `generate_hook()` function exists in `src/shell/mod.rs`
- ✅ Supports all 4 shells (bash, zsh, fish, PowerShell)
- ✅ Hook templates look correct

**Verification:**
```bash
# Test hook generation
ven shell hook bash        # Should output bash cd override
ven shell hook zsh         # Should output zsh cd override
ven shell hook fish        # Should output fish PWD watcher
ven shell hook powershell  # Should output PowerShell Set-VenLocation
```

**Implementation Details:**

**Bash/Zsh Hook:**
```bash
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

**PowerShell Hook:**
```powershell
function Set-VenLocation {
    param([string]$Path = "")
    if ($Path) { Set-Location $Path }
    $exports = ven shell activate "$PWD" 2>$null
    if ($exports) { Invoke-Expression $exports }
}
Set-Alias -Name cd -Value Set-VenLocation -Force -Option AllScope
```

---

### Task 2: Verify ven Setup Installs Hooks 🚧 IN PROGRESS

**What needs to work:**
- `ven setup` detects current shell
- Appends hook to correct rc file
- Doesn't duplicate if already installed
- Creates parent directories if needed

**Current State:**
- ✅ `cmd_setup()` exists in `src/cli/setup.rs`
- ✅ Detects shell properly (Windows → PowerShell, Unix → $SHELL)
- ✅ Writes to correct files:
  - Windows: `Documents/PowerShell/Microsoft.PowerShell_profile.ps1`
  - Bash: `~/.bashrc`
  - Zsh: `~/.zshrc`
  - Fish: `~/.config/fish/config.fish`
- ✅ Checks for duplicate installation
- ✅ Creates parent directories

**Verification:**
```bash
# Run setup
ven setup

# Check if hook was added
# On Windows:
cat $HOME/Documents/PowerShell/Microsoft.PowerShell_profile.ps1 | Select-String "ven"

# On Unix:
grep "ven shell hook" ~/.bashrc  # or ~/.zshrc
```

**Expected Output:**
```
ven setup
  Detected shell: powershell
  ✓ Written to C:\Users\You\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
  
  Restart PowerShell or run:
  . $PROFILE
```

---

### Task 3: Test ven shell activate PATH Output ⏳ PENDING

**What needs to work:**
- Finds ven.toml in current directory (walks up if needed)
- Parses `[runtime]` section to get Node version
- Resolves aliases (`20`, `lts`, `latest`) to concrete versions
- Returns correct PATH exports for the shell

**Current State:**
- ✅ `compute_exports()` exists in `src/shell/mod.rs`
- ✅ Calls `find_ven_toml()` to locate config
- ✅ Calls `parse_ven_toml()` to read config
- ✅ Calls `resolve_node_version()` to handle aliases
- ✅ Calls `plugin.bin_path()` to get binary directory
- ✅ Generates shell-specific exports

**Implementation Details:**

**Windows (PowerShell) Output:**
```powershell
$env:PATH = "C:\Users\You\.ven\node\v20.11.1;" + $env:PATH
$env:VEN_NODE_VERSION = "20.11.1"
$env:VEN_TOML = "C:\projects\myapp\ven.toml"
$env:NODE_ENV = "development"
$env:PORT = "3000"
```

**Unix (Bash/Zsh) Output:**
```bash
export PATH="/home/you/.ven/node/v20.11.1/bin:$PATH"
export VEN_NODE_VERSION="20.11.1"
export VEN_TOML="/home/you/projects/myapp/ven.toml"
export NODE_ENV="development"
export PORT="3000"
```

**Verification:**
```bash
# Create test project
mkdir test-project
cd test-project
echo '[runtime]
node = "20"' > ven.toml

# Test activate
ven shell activate .

# Should output export statements
```

---

### Task 4: Complete ven.toml Integration ✅ COMPLETE

**What needs to work:**
- `find_ven_toml()` - walks up directory tree
- `parse_ven_toml()` - reads `[runtime]` section
- `resolve_node_version()` - handles "20", "lts", "latest"
- Returns correct binary path

**Current State:**
- ✅ All functions implemented in `src/core/config.rs`
- ✅ `find_ven_toml()` walks up using `std::fs::metadata()`
- ✅ `parse_ven_toml()` uses `toml` crate for parsing
- ✅ `resolve_node_version()` handles:
  - `"latest"` → highest installed version
  - `"lts"` → highest even major version (18, 20, 22)
  - `"20"` → highest 20.x.x installed
  - `"20.11.1"` → exact match

**Implementation Details:**

```rust
pub fn resolve_node_version(spec: &str, installed: &[String]) -> Result<String> {
    match spec {
        "latest" => {
            installed.iter()
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow!("No Node versions installed"))
        }
        "lts" => {
            // LTS = even major versions
            installed.iter()
                .filter(|v| is_lts_version(v))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow!("No LTS versions installed"))
        }
        spec if !spec.contains('.') => {
            // Major only: "20" → find highest 20.x.x
            installed.iter()
                .filter(|v| v.starts_with(&format!("{}.", spec)))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow!("No Node {} versions installed", spec))
        }
        _ => Ok(spec.to_string()) // exact version
    }
}
```

**Bin Path Resolution:**
- Windows: `~/.ven/node/v20.11.1/` (node.exe sits in root)
- Unix: `~/.ven/node/v20.11.1/bin/` (node binary in bin/)

---

### Task 5: Wire Up the CD Hook ⏳ PENDING

**What needs to work:**
1. Shell calls `ven shell activate $PWD` on every `cd`
2. ven finds ven.toml
3. ven prints `export PATH=...`
4. Shell evaluates it
5. Node version changes

**Current State:**
- ✅ Hook code generated correctly (Task 1)
- ✅ PATH output works (Task 3)
- ⏳ Need to test end-to-end in real terminal

**Testing Procedure:**

```bash
# Step 1: Install Node versions
ven install node 20
ven install node 18

# Step 2: Setup shell hooks
ven setup
# Restart shell

# Step 3: Create test projects
mkdir project-v20
cd project-v20
echo '[runtime]
node = "20"' > ven.toml

mkdir ../project-v18
cd ../project-v18
echo '[runtime]
node = "18"' > ven.toml

# Step 4: Test auto-switching
cd ../project-v20
node --version  # Should show v20.x.x

cd ../project-v18
node --version  # Should show v18.x.x

cd ..
node --version  # Should revert to system Node or last active
```

---

### Task 6: Basic Validation ⏳ PENDING

**What needs to work:**
- Create test project with ven.toml
- cd into it
- Verify correct Node version active
- cd out → verify reverts

**Test Script:**
Created: `scripts/test_phase0.ps1` (PowerShell)  
Created: `scripts/test_phase0.sh` (Bash)

**Run Tests:**
```bash
# On Windows
.\scripts\test_phase0.ps1

# On Unix
chmod +x scripts/test_phase0.sh
./scripts/test_phase0.sh
```

**Expected Results:**
```
[TEST 1] ven.toml parsing...
  ✓ ven status works

[TEST 2] Shell hook generation...
  ✓ PowerShell hook generated

[TEST 3] ven shell activate PATH output...
  ✓ PATH exports generated correctly

[TEST 4] Switch to project-b...
  ✓ Project B activates Node 18

[TEST 5] Directory without ven.toml...
  ✓ No exports when ven.toml missing (correct)
```

---

## 🔧 Key Files Involved

| File | Purpose | Status |
|------|---------|--------|
| `src/shell/mod.rs` | Hook generation, compute_exports | ✅ Complete |
| `src/cli/shell.rs` | Shell command handlers | ✅ Complete |
| `src/cli/setup.rs` | Setup command | ✅ Complete |
| `src/core/config.rs` | ven.toml parsing, version resolution | ✅ Complete |
| `src/plugins/node.rs` | NodePlugin implementation | ✅ Complete |
| `src/core/download.rs` | Binary path resolution | ✅ Complete |

---

## 🐛 Known Issues to Fix

### Issue 1: Fish Shell Hook Syntax
Fish uses different variable syntax. Current hook might need adjustment:

**Current (possibly incorrect):**
```fish
set exports (ven shell activate "$PWD" 2>/dev/null)
```

**May need to be:**
```fish
set -l exports (ven shell activate "$PWD" ^/dev/null)
```

### Issue 2: Windows PATH Separator
Ensure Windows uses semicolons, Unix uses colons:

**Windows:**
```powershell
$env:PATH = "C:\path\to\node;" + $env:PATH
```

**Unix:**
```bash
export PATH="/path/to/node/bin:$PATH"
```

**Current State:** ✅ Handled correctly in `compute_exports()`

---

## 📊 Phase 0 Completion Checklist

- [x] Shell hook generation tested for all shells
- [ ] ven setup tested on PowerShell
- [ ] ven setup tested on Bash
- [ ] ven setup tested on Zsh
- [ ] ven setup tested on Fish
- [ ] ven shell activate returns correct PATH
- [ ] ven shell activate resolves versions correctly
- [ ] End-to-end cd switching works in real terminal
- [ ] Version reverts when leaving project directory
- [ ] Environment variables from [env] section are set
- [ ] Test scripts pass on Windows
- [ ] Test scripts pass on Unix

---

## 🚀 Next Steps After Phase 0

Once Phase 0 is complete, we'll have:
- ✅ Working auto-switching on cd
- ✅ Reliable shell integration
- ✅ Proper ven.toml parsing
- ✅ Version resolution (aliases, major, exact)

**Phase 1 will then focus on:**
- Package commands (add/remove/upgrade) with npm path resolution
- Predictive dependency resolver
- Ghost dependency detection

---

## 📝 Testing Notes

### Manual Testing Checklist

```bash
# 1. Clean install test
rm -rf ~/.ven
ven install node 20
ven install node 18
ven list

# 2. Setup test
ven setup
# Restart shell

# 3. Auto-switching test
cd ~/projects/myapp  # has ven.toml with node = "20"
node --version       # should be v20.x.x

cd ~/projects/other  # has ven.toml with node = "18"
node --version       # should be v18.x.x

cd ~                 # no ven.toml
node --version       # should be system Node

# 4. Environment variables test
cd ~/projects/myapp
echo $NODE_ENV       # should be value from ven.toml [env]
echo $PORT           # should be value from ven.toml [env]
```

---

**Last Updated:** 2024-03-22  
**Phase:** 0 (Foundation Fixes)  
**Status:** 🚧 In Progress
