# `ven shell` - Auto-Monitoring Hook Documentation

## How It Works

The `ven shell` system has TWO modes:

### 1. Manual Activation (One-Time)
```powershell
ven shell activate .
```
- Outputs shell commands
- User must eval them
- **Does NOT monitor directory changes**
- Good for scripts/CI-CD

### 2. Automatic Monitoring (Hook) ✅ RECOMMENDED
```powershell
# Setup once
Invoke-Expression (ven shell hook powershell)

# Now cd automatically switches Node versions
cd project-a  # Auto: activates Node for project-a
cd project-b  # Auto: switches to project-b's Node
cd /tmp       # Auto: removes ven paths, uses global Node
```

---

## What the Hook Does

### When You `cd` Into a Directory:

1. **Checks if directory changed** (optimization)
2. **Searches for ven.toml** in current and parent directories
3. **If ven.toml found:**
   - Resolves Node version (e.g., "20" → "20.20.2")
   - Updates PATH to use ven's Node
   - Sets VEN_NODE_VERSION and VEN_TOML
4. **If NO ven.toml:**
   - **Removes ven PATH entries**
   - **Restores original system PATH**
   - **Unsets VEN_NODE_VERSION and VEN_TOML**
   - Falls back to global Node

### Directory Navigation Flow:

```
User starts shell
    ↓
Hook activates current directory's ven.toml (if exists)
    ↓
User runs: cd project-a
    ↓
Hook detects directory change
    ↓
Hook finds project-a/ven.toml
    ↓
Hook activates Node 20 for project-a
    ↓
User runs: cd ../project-b
    ↓
Hook detects directory change
    ↓
Hook finds project-b/ven.toml
    ↓
Hook switches to Node 18 for project-b
    ↓
User runs: cd /tmp
    ↓
Hook detects directory change
    ↓
Hook finds NO ven.toml
    ↓
Hook REMOVES ven paths, restores global Node
```

---

## Setup Instructions

### PowerShell (Windows)

**Option 1: Current Session Only**
```powershell
Invoke-Expression (ven shell hook powershell)
```

**Option 2: Persistent (Add to Profile)**
```powershell
# Open PowerShell profile
notepad $PROFILE

# Add this line:
Invoke-Expression (ven shell hook powershell)

# Save and restart PowerShell
```

### Bash (Linux/Mac)

**Option 1: Current Session Only**
```bash
eval "$(ven shell hook bash)"
```

**Option 2: Persistent**
```bash
# Add to ~/.bashrc
echo 'eval "$(ven shell hook bash)"' >> ~/.bashrc
source ~/.bashrc
```

### Zsh (Mac)

```bash
# Add to ~/.zshrc
echo 'eval "$(ven shell hook zsh)"' >> ~/.zshrc
source ~/.zshrc
```

---

## Testing the Hook

### Test 1: Basic Activation

```powershell
# Setup hook
Invoke-Expression (ven shell hook powershell)

# Check initial state
node -v  # Shows global Node (e.g., v24.6.0)

# Move to project with ven.toml
cd d:/projects/software/ven/example

# Hook auto-activates!
node -v  # Now shows v20.20.2 (from ven.toml)
```

### Test 2: Switching Projects

```powershell
# Currently in project-a (Node 20)
node -v  # v20.20.2

# Switch to project-b (different ven.toml)
cd ../sample

# Hook auto-switches!
node -v  # v25.9.0 (from sample/ven.toml)
```

### Test 3: Leaving Project (No ven.toml)

```powershell
# Currently in project (Node 20)
node -v  # v20.20.2

# Move to directory without ven.toml
cd C:/tmp

# Hook removes ven paths!
node -v  # v24.6.0 (back to global Node)
```

### Test 4: Verify PATH Changes

```powershell
# Before hook
$env:PATH -split ';' | Select-String 'node'
# Output: C:\Program Files\nodejs\

# After cd to project
cd d:/projects/software/ven/example
$env:PATH -split ';' | Select-String 'node' | Select-Object -First 1
# Output: C:\Users\...\node\20.20.2 (ven's Node is first!)

# After cd away from project
cd C:/tmp
$env:PATH -split ';' | Select-String 'node' | Select-Object -First 1
# Output: C:\Program Files\nodejs\ (back to global)
```

---

## Hook Features

### ✅ What It Does:

1. **Monitors `cd` commands**
   - Overrides the `cd` command
   - Detects directory changes
   - Automatically searches for ven.toml

2. **Smart Activation**
   - Only re-activates when directory changes (performance)
   - Finds ven.toml in current or parent directories
   - Resolves Node versions correctly

3. **Automatic Cleanup**
   - **Removes ven PATH** when leaving projects
   - **Restores original PATH** (no stale paths)
   - **Unsets VEN variables** (VEN_NODE_VERSION, VEN_TOML)

4. **Error Handling**
   - Silent errors (2>$null)
   - Doesn't break if ven.toml is invalid
   - Gracefully falls back to global Node

### ❌ What It Doesn't Do:

- Manual `ven shell activate` (use hook instead)
- Persistent across shell restarts (need to add to profile)
- Works with `pushd/popd` (only monitors `cd`)

---

## How PATH Management Works

### Original PATH (Before Hook):
```
C:\Program Files\nodejs\
C:\Windows\system32
...
```

### After `cd` to Project with ven.toml:
```
C:\Users\Bhuwan\.ven\node\20.20.2  ← ADDED (first)
C:\Program Files\nodejs\
C:\Windows\system32
...
```

### After `cd` Away (No ven.toml):
```
C:\Program Files\nodejs\  ← BACK TO ORIGINAL
C:\Windows\system32
...
```

**Key Point:** The hook **restores the original PATH** when leaving projects, so you never have stale ven paths!

---

## Troubleshooting

### Issue: Hook not working

**Solution:**
```powershell
# 1. Check if hook is loaded
Get-Alias cd
# Should show: Set-VenLocation

# 2. Reload hook
Invoke-Expression (ven shell hook powershell)

# 3. Verify ven is in PATH
where.exe ven
```

### Issue: Node version not switching

**Solution:**
```powershell
# 1. Check if ven.toml exists
Get-ChildItem -Recurse -Filter ven.toml

# 2. Check ven shell activate output
ven shell activate .

# 3. Check PATH
$env:PATH -split ';' | Select-String 'ven'

# 4. Check which node is being used
Get-Command node -All | Select-Object Source
```

### Issue: Old ven paths stuck in PATH

**Solution:**
```powershell
# 1. cd to a directory without ven.toml
cd C:/tmp

# 2. Hook should auto-clean
# If not, manually restore:
$env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine")
```

---

## Comparison: Manual vs Hook

| Feature | Manual (`ven shell activate`) | Hook (`ven shell hook`) |
|---------|-------------------------------|-------------------------|
| Auto-switches on cd | ❌ No | ✅ Yes |
| Removes paths when leaving | ❌ No | ✅ Yes |
| Requires eval | ✅ Yes | ❌ No (one-time setup) |
| Persistent | ❌ No | ✅ Yes (if added to profile) |
| Good for scripts | ✅ Yes | ❌ No |
| Good for interactive | ❌ No | ✅ Yes |

---

## Recommended Workflow

### For Daily Development:

```powershell
# 1. Add to PowerShell profile (one-time)
Invoke-Expression (ven shell hook powershell)

# 2. Use normally
cd my-project    # Auto: activates Node
cd another-proj  # Auto: switches Node
cd /tmp          # Auto: uses global Node
```

### For Scripts/CI-CD:

```powershell
# Manual activation (no hook)
ven shell activate . | Invoke-Expression
node -v  # Uses ven's Node
```

---

## Status: PRODUCTION READY ✅

The hook now properly:
- ✅ Monitors directory changes
- ✅ Activates ven.toml when found
- ✅ **Removes ven paths when leaving projects**
- ✅ Restores original PATH
- ✅ Prevents stale PATH entries
- ✅ Works across all shells (PowerShell, Bash, Zsh, Fish)

---

**Last Updated:** 2026-04-26  
**Status:** ✅ COMPLETE - Auto-monitoring with cleanup
