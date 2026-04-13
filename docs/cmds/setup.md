# ven setup

Install the shell hook for automatic Node.js version switching on directory change.

## Overview

The `setup` command configures your shell to automatically activate the correct Node.js version when you `cd` into a directory with a `ven.toml` file.

This is ven's **auto-switching magic** - the same UX as pyenv, nvm, or rbenv.

## Usage

```bash
ven setup
```

### Examples

#### First-Time Setup

```bash
ven setup
```

**Output (PowerShell):**
```
  → ven setup
  Detected shell: powershell
  ✓ Written to C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1

  Restart PowerShell or run:
  . $PROFILE
```

**Output (bash/zsh):**
```
  → ven setup
  Detected shell: zsh
  ✓ Written to /Users/you/.zshrc

  Restart your shell or run:
  source /Users/you/.zshrc
```

#### Already Configured

```bash
ven setup
```

**Output:**
```
  → ven setup
  Detected shell: powershell
  ✓ Shell hook already installed in C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
```

---

## How It Works

### Shell Detection

ven automatically detects your shell:

| Platform | Detection Method | Shell |
|----------|------------------|-------|
| Windows | `PSModulePath` env var | PowerShell |
| macOS/Linux | `$SHELL` env var | bash/zsh/fish |

### Hook Installation

ven appends a small script to your shell's configuration file:

**PowerShell:**
```powershell
# ven shell hook
Invoke-Expression (& ven shell hook powershell | Out-String)
```

**bash/zsh:**
```bash
# ven shell hook
eval "$(ven shell hook bash)"
```

**fish:**
```fish
# ven shell hook (fish)
ven shell hook fish | source
```

---

## Auto-Switching Behavior

### What Happens When You `cd`

```bash
cd /home/user/projects/api/
```

**Process:**
1. Shell hook detects directory change
2. Calls `ven shell activate $PWD`
3. ven searches for `ven.toml` (walks up directory tree)
4. If found:
   - Reads `runtime.node` version
   - Resolves to installed version
   - Calculates binary path
   - Outputs PATH modification
5. Shell evaluates the exports
6. Node.js version is now active

### Example

**Directory structure:**
```
/home/user/projects/
├── api/
│   └── ven.toml  # node = "20.11.0"
└── web/
    └── ven.toml  # node = "22.3.0"
```

**Session:**
```bash
cd /home/user/projects/api/
node --version
# v20.11.0  ← Auto-activated!

cd ../web/
node --version
# v22.3.0   ← Auto-switched!

cd /home/user/
node --version
# v18.20.2  ← Back to global default
```

---

## Shell-Specific Configuration

### PowerShell (Windows)

**Config file:**
```
C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
```

**Hook code:**
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
- Overrides `cd` command
- Calls `Set-VenLocation` on every directory change
- Runs `ven shell activate` to get PATH exports
- Uses `Invoke-Expression` to apply changes

**PATH syntax (Windows):**
```powershell
$env:PATH = "C:\Users\you\.ven\node\20.11.0;" + $env:PATH
$env:VEN_NODE_VERSION = "20.11.0"
$env:VEN_TOML = "C:\Users\you\projects\api\ven.toml"
```

---

### bash (Linux/macOS)

**Config file:**
```
~/.bashrc
```

**Hook code:**
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
- Overrides `cd` command
- Calls `__ven_activate` after every `cd`
- Captures output from `ven shell activate`
- Uses `eval` to apply exports

**PATH syntax (Unix):**
```bash
export PATH="/home/you/.ven/node/20.11.0/bin:$PATH"
export VEN_NODE_VERSION="20.11.0"
export VEN_TOML="/home/you/projects/api/ven.toml"
```

---

### zsh (macOS)

**Config file:**
```
~/.zshrc
```

**Hook code:** Same as bash (fully compatible)

---

### fish (Linux/macOS)

**Config file:**
```
~/.config/fish/config.fish
```

**Hook code:**
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
- Uses `--on-variable PWD` trigger
- Automatically runs when `$PWD` changes
- No need to override `cd`

**PATH syntax (fish):**
```fish
set -gx PATH "/home/you/.ven/node/20.11.0/bin" $PATH
set -gx VEN_NODE_VERSION "20.11.0"
set -gx VEN_TOML "/home/you/projects/api/ven.toml"
```

---

## Command Reference

### Syntax

```bash
ven setup
```

### Arguments

None - automatic detection and configuration.

### Options

None - simple one-time setup command.

---

## Environment Variables

### Set by Shell Hook

| Variable | Description | Example |
|----------|-------------|---------|
| `PATH` | Prepended with Node binary path | `~/.ven/node/20.11.0/bin:$PATH` |
| `VEN_NODE_VERSION` | Currently active Node version | `20.11.0` |
| `VEN_TOML` | Path to active ven.toml | `/path/to/project/ven.toml` |

### Set from ven.toml [env] Section

```toml
[env]
NODE_ENV = "development"
PORT = "3000"
```

**Becomes:**
```bash
export NODE_ENV="development"
export PORT="3000"
```

---

## Manual Activation

### Without Shell Hook

If you don't want to modify your shell config:

```bash
# Manual activation
eval "$(ven shell activate $PWD)"

# Or create alias
alias ven-activate='eval "$(ven shell activate $PWD)"'

# Use when needed
cd myproject/
ven-activate
```

### Test Without Installing

```bash
# See what would be exported
ven shell activate $PWD

# Output (if ven.toml exists):
export PATH="/home/you/.ven/node/20.11.0/bin:$PATH"
export VEN_NODE_VERSION="20.11.0"
export VEN_TOML="/home/you/myproject/ven.toml"
```

---

## Use Cases

### 1. Developer Workstation Setup

```bash
# After installing ven
ven setup
source ~/.zshrc

# Done! Auto-switching enabled
```

### 2. CI/CD Environment

```bash
# In CI script
ven setup
source ~/.bashrc

cd project/
node --version  # Automatically correct version
npm install
npm test
```

### 3. Monorepo Development

```bash
# Navigate between projects
cd monorepo/frontend/
node --version  # 20.11.0

cd ../backend/
node --version  # 22.3.0

cd ../mobile/
node --version  # 18.20.2
```

### 4. Team Consistency

```bash
# Everyone runs same setup
git clone <repo>
cd project/
ven setup
source ~/.zshrc

# Now everyone has correct Node version
node --version  # From ven.toml
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/setup.rs`](../../src/cli/setup.rs) (69 lines)
- **Hook generation**: [`src/shell/mod.rs`](../../src/shell/mod.rs)

### Key Functions

```rust
// CLI layer
cmd_setup()

// Shell module
detect_shell()           // Auto-detect running shell
generate_hook(shell)     // Generate shell-specific hook
compute_exports(dir)     // Calculate PATH exports
```

### Shell Detection Logic

```rust
pub fn detect_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        if std::env::var("PSModulePath").is_ok() {
            return "powershell".to_string();
        }
        return "powershell".to_string();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shell_path = std::env::var("SHELL").unwrap_or_default();
        std::path::Path::new(&shell_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bash")
            .to_string()
    }
}
```

---

## Troubleshooting

### Hook Not Working

**Problem:** Node version doesn't change on `cd`

**Solutions:**

1. **Reload shell config:**
```bash
# PowerShell
. $PROFILE

# bash/zsh
source ~/.bashrc  # or source ~/.zshrc

# fish
source ~/.config/fish/config.fish
```

2. **Check hook was added:**
```bash
# PowerShell
cat $PROFILE | Select-String "ven shell hook"

# bash/zsh
grep "ven shell hook" ~/.bashrc
```

3. **Manual test:**
```bash
ven shell activate $PWD
# Should output export commands
```

### Wrong Shell Detected

**Problem:** ven detects wrong shell

**Solution:**
```bash
# Check detected shell
echo $SHELL

# Force specific shell (modify setup script manually)
# Edit ~/.bashrc and add:
eval "$(ven shell hook bash)"
```

### Hook Already Exists

**Problem:** "Shell hook already installed" but not working

**Solution:**
```bash
# Check for duplicate hooks
grep -n "ven shell hook" ~/.bashrc

# Remove duplicates, keep one
nano ~/.bashrc

# Reload
source ~/.bashrc
```

### PATH Not Updating

**Problem:** Node version shows old version

**Solution:**
```bash
# Check current PATH
echo $PATH

# Verify ven path is there
echo $PATH | grep -o "$HOME/.ven/node/[^:]*"

# Check active version
echo $VEN_NODE_VERSION

# Manually activate
eval "$(ven shell activate $PWD)"
```

---

## Uninstall Hook

### Remove from Shell Config

```bash
# PowerShell
notepad $PROFILE
# Delete lines containing "ven shell hook"

# bash/zsh
nano ~/.bashrc
# Delete lines containing "ven shell hook"
source ~/.bashrc

# fish
nano ~/.config/fish/config.fish
# Delete lines containing "ven shell hook"
```

### Or Use One-Liner

```bash
# bash/zsh
sed -i '/# ven shell hook/,+3d' ~/.bashrc
source ~/.bashrc
```

---

## Best Practices

### 1. Run Setup Once

```bash
# After installing ven
ven setup
source ~/.zshrc

# Don't run multiple times (checks for duplicates)
```

### 2. Test After Setup

```bash
# Create test project
mkdir /tmp/test-ven && cd /tmp/test-ven
ven init --node 20
ven install node 20

cd /tmp/
cd /tmp/test-ven/
node --version  # Should show 20.x.x
```

### 3. Keep ven.toml Updated

```bash
# When changing Node version
# Edit ven.toml
nano ven.toml

# Auto-switches on next cd
cd ..
cd project/
node --version  # New version
```

### 4. Debug Issues

```bash
# See what ven would export
ven shell activate $PWD

# Check if ven.toml found
find . -name ven.toml

# Verify Node installed
ven list
```

---

## Related Commands

- [`ven init`](init.md) - Create ven.toml
- [`ven install`](install.md) - Install Node.js versions
- [`ven status`](status.md) - Check current config
- `ven shell activate` - Manual activation (internal)

---

## Next Steps

After setup:

```bash
# Create a project
ven init --template

# Install Node version
ven install node 20

# Test auto-switching
cd ..
cd project/
node --version  # Should auto-switch!

# Add packages
ven add express
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
