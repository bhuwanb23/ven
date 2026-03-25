# Phase 1 Testing & Verification Plan

## Status: Ready for Testing (Rust installation required)

---

## 🔧 Prerequisites

Before running tests, ensure the following are installed:

1. **Rust Toolchain** (stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   # Or on Windows: winget install Rustlang.Rust.MSVC
   ```

2. **fnm** (Fast Node Manager) - Required for Node.js operations
   ```bash
   curl -fsSL https://fnm.vercel.app/install | bash
   ```

3. **Node.js** (any version, for testing)
   ```bash
   fnm install node --latest
   ```

---

## ✅ Test Checklist

### 1. Unit Tests (Automated)

Run all unit tests:
```bash
cargo test --all
```

**Expected outcomes:**
- ✅ All tests pass (0 failures)
- ✅ No compilation warnings
- ✅ Code coverage includes:
  - `core::config` - TOML parsing
  - `core::packages` - NPM compatibility logic
  - `plugins::node` - Version resolution

**Known test to verify:**
- `test_parse_valid_ven_toml` - Parses example config correctly
- `test_find_ven_toml` - Walks up directory tree correctly
- `test_node_version_satisfies_basic` - Engine requirement checking
- `test_semver_cmp` - Semantic version comparison

---

### 2. CLI Commands (Manual Testing)

Create a test project directory:
```bash
mkdir -p /tmp/ven-test-project
cd /tmp/ven-test-project
```

#### Test 2.1: `ven init`
```bash
ven init
```
**Expected:**
- Creates `ven.toml` with current Node version detected
- File contains `[runtime]` and `[packages]` sections

#### Test 2.2: `ven status`
```bash
ven status
```
**Expected:**
- Shows current directory path
- Displays Node version from `ven.toml`
- Lists number of declared packages

#### Test 2.3: `ven install node <version>`
```bash
ven install node 20.11.0
```
**Expected:**
- Downloads Node.js 20.11.0 via fnm
- Shows progress messages
- Version appears in `ven list`

#### Test 2.4: `ven list`
```bash
ven list node
```
**Expected:**
- Shows all installed Node versions
- Marks currently active version with `*`

#### Test 2.5: `ven add <package>`
```bash
ven add express
```
**Expected:**
- Checks compatibility with current Node version
- Installs compatible version via npm
- Updates `ven.toml` with package entry

#### Test 2.6: `ven upgrade <package>`
```bash
ven upgrade express
```
**Expected:**
- Shows current vs latest compatible version
- Displays changelog link
- With `--apply`: actually upgrades

#### Test 2.7: `ven remove <package>`
```bash
ven remove express
```
**Expected:**
- Warns about dependent packages
- Asks for confirmation
- Uninstalls via npm

---

### 3. Shell Hook Integration

#### Test 3.1: Generate hook code
```bash
ven shell hook bash
```
**Expected:**
- Prints shell function definitions
- Includes `__ven_activate` function
- Overrides `cd` command

#### Test 3.2: Install shell hook
```bash
ven setup
```
**Expected:**
- Detects current shell (bash/zsh/fish)
- Appends hook to `~/.bashrc` or `~/.zshrc`
- Shows instructions to reload shell

#### Test 3.3: Auto-activation on cd
```bash
# Create two projects with different Node versions
mkdir -p /tmp/project-a /tmp/project-b

# Configure project-a with Node 20
cd /tmp/project-a
ven init --node 20

# Configure project-b with Node 18
cd /tmp/project-b
ven init --node 18

# Test switching
cd /tmp/project-a
node --version  # Should show v20.x.x

cd /tmp/project-b
node --version  # Should switch to v18.x.x
```

**Expected:**
- PATH updates automatically on directory change
- Correct Node version activates silently
- Environment variables set correctly

---

### 4. Configuration System

#### Test 4.1: ven.toml parsing
File: `example/ven.toml` (already created)
```toml
[runtime]
node = "20.11.1"

[packages]
express = "^4.18.2"
react = "18.2.0"

[env]
NODE_ENV = "development"
PORT = "3000"
```

**Verify:**
- ✅ All fields parse correctly
- ✅ Optional fields handled (dev_packages)
- ✅ Invalid TOML produces clear error

#### Test 4.2: Directory walk-up
```bash
cd example/a/b/c  # nested structure
ven status
```
**Expected:**
- Finds `ven.toml` in parent `example/` directory
- Activates correct configuration

---

### 5. Compatibility Checking

#### Test 5.1: Package engine requirements
```bash
# Try to install a package requiring Node 18+ when using Node 16
ven init --node 16
ven install node 16.0.0
ven add some-package-requiring-node-18
```
**Expected:**
- Warns about incompatibility
- Suggests upgrading Node version
- Prevents broken install

#### Test 5.2: Version resolution
Test these version specifiers:
- `"latest"` → resolves to newest
- `"lts"` → resolves to latest LTS (even major: 18, 20, 22)
- `"20"` → resolves to latest 20.x.x
- `"20.11.0"` → exact version

---

### 6. Edge Cases & Error Handling

Test these scenarios:

1. **No ven.toml present**
   ```bash
   cd /tmp
   ven status
   ```
   **Expected:** Clear message: "No ven.toml found"

2. **Invalid Node version**
   ```bash
   ven install node 99.99.99
   ```
   **Expected:** Error: "Failed to install Node 99.99.99"

3. **Non-existent package**
   ```bash
   ven add this-package-does-not-exist
   ```
   **Expected:** Error: "Package not found on npm"

4. **Missing fnm**
   ```bash
   # Temporarily rename fnm binary
   ven install node 20.0.0
   ```
   **Expected:** Error: "fnm not found. Install it: ..."

5. **Corrupted ven.toml**
   ```toml
   [runtime]
   node = invalid syntax here!!!
   ```
   **Expected:** Error: "Invalid ven.toml at [path]: [details]"

---

## 🐛 Known Issues to Verify

### Issue 1: Duplicate Config Structs
**Location:** `src/core/mod.rs` vs `src/core/config.rs`

Both files define `VenConfig` and `RuntimeConfig` but with **different structures**:

**mod.rs (lines 13-24):**
```rust
pub struct VenConfig {
    pub runtime: RuntimeConfig,
    pub packages: Option<HashMap<String, String>>,
    pub dev_packages: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

pub struct RuntimeConfig {
    pub node: Option<String>,
}
```

**config.rs (lines 7-19):**
```rust
pub struct VenConfig {
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub packages: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub struct RuntimeConfig {
    pub node: String,
}
```

**⚠️ This will cause compilation errors!** Need to:
- Keep only ONE definition (recommend keeping `config.rs` version)
- Update imports in other modules
- Ensure tests use correct struct

### Issue 2: Missing Exports in `core/mod.rs`
Line 7 exports:
```rust
pub use packages::{fetch_npm_info, find_compatible_version, npm_install};
```

But `cli/mod.rs` line 321 uses:
```rust
use crate::core::{load_config, packages::*};
```

This should work, but verify all functions are properly exported.

---

## 📊 Phase 1 Completion Criteria

Mark each as complete when verified:

- [ ] ✅ **Build System**
  - [ ] `cargo build` succeeds with 0 errors
  - [ ] `cargo clippy` shows no warnings
  - [ ] `cargo fmt --check` passes
  - [ ] CI pipeline runs green

- [ ] ✅ **Core Functionality**
  - [ ] `ven.toml` parsing works for all valid configs
  - [ ] Error messages are clear for invalid configs
  - [ ] Directory walk-up finds nearest `ven.toml`

- [ ] ✅ **Version Management**
  - [ ] Can install multiple Node versions
  - [ ] Can list installed versions
  - [ ] Version aliases resolve correctly (lts, latest, major)

- [ ] ✅ **Shell Integration**
  - [ ] Hook generates valid shell code
  - [ ] Setup installs hook to correct rc file
  - [ ] Auto-activation works on `cd`
  - [ ] PATH updates correctly

- [ ] ✅ **Package Management**
  - [ ] `ven add` checks compatibility before install
  - [ ] `ven remove` warns about dependents
  - [ ] `ven upgrade` shows available updates
  - [ ] `ven.toml` updates after package changes

- [ ] ✅ **Error Handling**
  - [ ] Graceful failure with helpful messages
  - [ ] No panics in normal operation
  - [ ] Edge cases handled (missing files, network errors)

---

## 🚀 Next Steps After Testing

Once Phase 1 is verified:

1. **Fix any bugs discovered during testing**
2. **Update README.md** with working examples
3. **Add integration tests** to automated suite
4. **Document known limitations** for Phase 2
5. **Plan Phase 2 features:**
   - Ghost dependency detection
   - Vulnerability monitoring (OSV + endoflife.date)
   - Lock file generation (`ven.lock`)
   - Python plugin implementation
   - Documentation pinning

---

## 📝 Test Results Template

Use this format to record results:

```
## Test Run: [DATE]
**Tester:** [Your name]
**Environment:** 
- OS: [Windows/Linux/Mac]
- Rust: [version]
- Node: [version]
- fnm: [version]

### Results:
- Unit Tests: [PASS/FAIL] - [X/Y tests passing]
- CLI Commands: [PASS/FAIL] - [Issues found]
- Shell Hook: [PASS/FAIL] - [Notes]
- Configuration: [PASS/FAIL] - [Notes]

### Bugs Found:
1. [Description]
2. [Description]

### Blockers:
- [What prevents further testing]
```

---

**Ready to begin testing once Rust is installed!**
