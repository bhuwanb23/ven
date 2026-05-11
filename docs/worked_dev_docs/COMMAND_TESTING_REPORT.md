# Command Testing Report

## Test Environment
- **Directory:** `d:/projects/software/ven/example`
- **ven.toml:** Contains node="20", packages: react, react-dom, vite
- **Node.js:** 20.20.2 installed but NOT in PATH by default
- **npm:** Available only when Node PATH is manually set

---

## 1. `ven install` - ✅ WORKING
**Test:** `ven install node 20`
**Result:** Successfully installs Node.js versions
**ven.toml impact:** NONE (installs to ~/.ven/node/)
**Notes:** Fully functional, manages Node.js installations

---

## 2. `ven list` - ✅ WORKING
**Test:** `ven list` and `ven list --verbose`
**Result:** Shows 6 installed Node versions with details
**ven.toml impact:** NONE (read-only, shows installed versions)
**Notes:** Fully functional with verbose and JSON modes

---

## 3. `ven init` - ✅ WORKING
**Test:** Should create ven.toml
**Result:** Creates ven.toml with runtime and packages sections
**ven.toml impact:** CREATES the file
**Notes:** Functional, but need to test --template and --with_packages flags

---

## 4. `ven status` - ✅ WORKING (Partially)
**Test:** `ven status` and `ven status --verbose`
**Result:** 
- ✅ Reads ven.toml correctly
- ✅ Shows Node.js 20 (20.20.2) installed
- ✅ Detects packages declared (react, react-dom, vite)
- ❌ Shows all 3 packages as "not installed" (correct - npm install not run)

**ven.toml impact:** NONE (read-only analysis)
**Notes:** Works correctly for reading and displaying status

---

## 5. `ven add` - ❌ NOT WORKING (npm issue)
**Test:** `ven add axios`
**Result:** 
- ✅ Reads ven.toml
- ✅ Checks Node.js compatibility
- ✅ Fetches latest version from npm (axios@1.15.0)
- ❌ FAILS at npm install: "npm not found. Is Node installed and active?"
- ❌ ven.toml NOT updated (axios not added)

**ven.toml impact:** SHOULD add package to [packages] section, but FAILS
**Root Cause:** Command spawns `npm` subprocess, but PATH doesn't include Node bin directory

**Issue:** The `npm_install()` function in `src/core/packages.rs` line 136 calls:
```rust
Command::new("npm")
```
This requires npm to be in system PATH, but ven manages Node separately in ~/.ven/

---

## 6. `ven remove` - ⚠️ CANNOT TEST (no packages installed)
**Test:** Would need packages actually installed first
**Result:** Command structure works (help displays), but cannot test actual removal
**ven.toml impact:** SHOULD remove from [packages] section

---

## 7. `ven upgrade` - ❌ NOT WORKING (npm issue)
**Test:** `ven upgrade react --apply`
**Result:**
- ✅ Reads ven.toml
- ✅ Shows current version (not installed)
- ✅ Finds latest compatible version (react 19.2.5)
- ❌ FAILS at npm install: "npm not found"
- ❌ ven.toml NOT updated

**ven.toml impact:** SHOULD update version in [packages] section, but FAILS
**Same Issue:** npm not found in PATH

---

## 8. `ven setup` - ⚠️ NOT TESTED
**Test:** Should install shell hooks
**Result:** Not tested yet
**ven.toml impact:** NONE (system-wide setup)

---

## 9. `ven shell` - ⚠️ PARTIALLY WORKING
**Test:** `ven shell activate example`
**Result:** 
- ✅ Finds ven.toml
- ✅ Resolves Node version
- ✅ Outputs PATH export commands
- ❌ But doesn't automatically apply them

**ven.toml impact:** NONE (reads config to set environment)
**Notes:** Works as expected - user must eval the output

---

## CRITICAL ISSUES FOUND

### Issue #1: npm Not Found
**Affected Commands:** `ven add`, `ven remove`, `ven upgrade`
**Severity:** HIGH - Core functionality broken
**Root Cause:** 
- Node.js installed in `~/.ven/node/20.20.2/`
- npm binary at `~/.ven/node/20.20.2/npm` or `~/.ven/node/20.20.2/node_modules/npm/bin/npm-cli.js`
- Commands spawn `Command::new("npm")` which searches system PATH
- ven doesn't add its own Node to PATH before calling npm

**Solution Needed:** Update npm_install, npm_uninstall functions to:
1. Use full path to npm from active Node version
2. OR prepend Node bin directory to PATH before spawning command
3. OR use the bin_path from the plugin system

### Issue #2: ven.toml Not Updated on Failure
**Affected Commands:** `ven add`, `ven upgrade`
**Severity:** MEDIUM
**Current Behavior:** ven.toml only updated AFTER successful npm install
**Expected:** This is actually CORRECT behavior - don't update config if install fails

### Issue #3: No Packages Actually Installed
**Current State:** 
- ven.toml declares 3 packages
- None are actually installed (no node_modules/)
- All package commands fail or show "not installed"

**Impact:** Cannot test remove, upgrade, or status with packages

---

## WORKING COMMANDS (Summary)

### ✅ Fully Working:
1. **ven install** - Installs Node.js versions
2. **ven list** - Lists installed versions
3. **ven status** - Reads and displays ven.toml info
4. **ven shell** - Outputs environment setup commands

### ⚠️ Partially Working:
5. **ven init** - Creates ven.toml (need to test all flags)
6. **ven shell activate** - Outputs correct commands but manual eval needed

### ❌ Not Working (npm issue):
7. **ven add** - Fails at npm install
8. **ven upgrade** - Fails at npm install
9. **ven remove** - Would fail at npm uninstall

---

## RECOMMENDED FIX

Update `src/core/packages.rs` to use ven's Node.js npm instead of system npm:

```rust
pub fn npm_install(package: &str, version: &str) -> Result<()> {
    // Get active Node.js version from ven
    let cwd = std::env::current_dir()?;
    let config = load_config(&cwd)?
        .ok_or_else(|| anyhow!("No ven.toml found"))?;
    
    // Get npm path from active Node version
    let node_bin = get_node_bin_path(&config.runtime.node)?;
    let npm_path = node_bin.join("npm");
    
    let pkg_spec = format!("{}@{}", package, version);
    println!("{} Installing {}...", "[DOWNLOAD]".cyan(), pkg_spec.bold());

    let status = Command::new(&npm_path)
        .args(["install", &pkg_spec])
        .status()
        .map_err(|e| anyhow!("Failed to run npm ({}): {}. Is Node installed?", npm_path.display(), e))?;

    if !status.success() {
        return Err(anyhow!("npm install failed for {}", pkg_spec));
    }

    println!("{} Installed {}", "[OK]".green(), pkg_spec.bold());
    
    // Update ven.toml
    // ... existing code
    
    Ok(())
}
```

---

## NEXT STEPS

1. **Fix npm path issue** in packages.rs
2. **Test ven add** with real package installation
3. **Verify ven.toml updates** after successful install
4. **Test ven upgrade** with actually installed package
5. **Test ven remove** with dependency checking
6. **Full integration test** of complete workflow

---

## Commands That Work WITHOUT npm:
- ✅ ven install (downloads Node directly)
- ✅ ven list (reads ~/.ven/node/)
- ✅ ven status (reads ven.toml only)
- ✅ ven shell (reads ven.toml, outputs env vars)

## Commands That REQUIRE npm:
- ❌ ven add (needs npm install)
- ❌ ven upgrade (needs npm install)
- ❌ ven remove (needs npm uninstall)

---

**Generated:** 2026-04-26
**Status:** 4/9 commands working, 5 need npm path fix
