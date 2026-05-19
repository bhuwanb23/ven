# Quick Start Guide - ven Phase 1 Testing

**TL;DR:** Get ven tested in 5 steps

---

## 🚀 Step-by-Step (Total Time: ~30 minutes)

### Step 1: Install Rust (10 min)

**Windows PowerShell:**
```powershell
winget install Rustlang.Rust.MSVC
```

**Or download from:** https://rustup.rs/

**Verify:**
```powershell
rustc --version
cargo --version
# Should show: rustc 1.x.x, cargo 1.x.x
```

---

### Step 2: Fix Critical Bug (15 min)

**Problem:** Duplicate struct definitions prevent compilation

**Quick Fix:**

1. Open `src/core/mod.rs`
2. **DELETE** lines 13-24 (the VenConfig and RuntimeConfig structs)
3. **DELETE** lines 30-112 (all the function implementations)
4. Keep only lines 1-8 and update line 6-7 to:

```rust
pub mod config;
pub mod packages;

pub use config::{VenConfig, RuntimeConfig, find_ven_toml, parse_ven_toml, load_config};
pub use packages::{fetch_npm_info, find_compatible_version, npm_install};
```

**Full details:** See `BUG_FIX_NEEDED.md`

---

### Step 3: Build Project (3 min)

```powershell
cd d:\projects\software\ven
cargo build --release
```

**Expected:** 
```
   Compiling ven v0.1.7
    Finished release [optimized] target(s)
```

**If errors:** Read error message, likely still struct conflicts

---

### Step 4: Run Tests (2 min)

```powershell
cargo test --all
```

**Expected:**
```
running 6 tests
test core::config::tests::test_parse_valid_ven_toml ... ok
test core::config::tests::test_find_ven_toml ... ok
test core::packages::tests::test_node_version_satisfies_basic ... ok
...
test result: ok. 6 passed; 0 failed
```

---

### Step 5: Test CLI Commands (5 min)

```powershell
# Create test project
mkdir C:\temp\ven-test
cd C:\temp\ven-test

# Test init
..\d\projects\software\ven\target\release\ven.exe init

# Check created file
cat ven.toml

# Test status
..\d\projects\software\ven\target\release\ven.exe status

# Test help
..\d\projects\software\ven\target\release\ven.exe --help
```

---

## ✅ Success Criteria

You know Phase 1 is working when:

- ✅ `cargo build` completes with 0 errors
- ✅ All 6 unit tests pass
- ✅ `ven init` creates valid ven.toml
- ✅ `ven status` shows current directory info
- ✅ `ven --help` displays all commands

---

## ❌ Troubleshooting

### "cargo not found"
Rust not installed or not in PATH. Restart terminal after installation.

### Compilation errors about duplicate definitions
Still have duplicate structs. Re-read `BUG_FIX_NEEDED.md` carefully.

### "fnm not found" warnings
Install fnm: `choco install fnm` (optional for basic testing)

### Tests fail
Read error messages. Likely parsing or path issues. Check file paths in tests.

---

## 📞 Next Steps After Success

1. **Run full test suite** from `TESTING_PLAN.md`
2. **Test shell hook** (requires bash/zsh/fish)
3. **Test package management** (requires Node.js + npm)
4. **Document any bugs** found
5. **Plan Phase 2** features

---

## 🎯 Reference Documents

| Document | Purpose |
|----------|---------|
| `BUG_FIX_NEEDED.md` | Critical fix instructions (READ FIRST) |
| `TESTING_PLAN.md` | Comprehensive test scenarios |
| `PHASE1_STATUS.md` | Full status report |
| `scripts/test_phase1.ps1` | Automated test runner (Windows) |
| `scripts/test_phase1.sh` | Automated test runner (Linux/Mac) |

---

**Estimated Total Time:** 30 minutes  
**Difficulty:** Beginner (follow instructions carefully)

Good luck! 🚀
