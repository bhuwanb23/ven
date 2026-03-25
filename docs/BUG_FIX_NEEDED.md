# ⚠️ CRITICAL BUG FIX REQUIRED BEFORE TESTING

## Issue: Duplicate Config Struct Definitions

### Problem Summary
You have **two conflicting definitions** of `VenConfig` and `RuntimeConfig` structs that will cause compilation errors.

---

## 🔴 Conflicting Definitions

### Definition 1: `src/core/mod.rs` (lines 13-24)

```rust
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct VenConfig {
    pub runtime:      RuntimeConfig,
    pub packages:     Option<HashMap<String, String>>,
    pub dev_packages: Option<HashMap<String, String>>,
    pub env:          Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct RuntimeConfig {
    pub node: Option<String>,   // Note: Option<String>
}
```

**Characteristics:**
- All fields are `Option<T>` (nullable)
- Has `dev_packages` field
- Uses `Default` trait

---

### Definition 2: `src/core/config.rs` (lines 7-19)

```rust
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct VenConfig {
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub packages: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RuntimeConfig {
    pub node: String,  // Note: String (required!)
}
```

**Characteristics:**
- `runtime.node` is required (not optional)
- `packages` and `env` use `#[serde(default)]` for empty HashMap
- No `dev_packages` field
- Has `PartialEq` instead of `Default`

---

## 🛠️ Recommended Fix

**Keep the `config.rs` version** and remove duplicate from `mod.rs`. Here's why:

1. ✅ Better TOML ergonomics with `#[serde(default)]`
2. ✅ Required `node` field ensures valid config
3. ✅ Used by existing tests in `config.rs`
4. ✅ More idiomatic Rust design

### Step-by-Step Fix

#### 1. Remove duplicate structs from `src/core/mod.rs`

Delete lines 13-24 from `mod.rs` completely.

#### 2. Update imports in `src/core/mod.rs`

Change line 6-7 from:
```rust
pub mod packages;
pub use packages::{fetch_npm_info, find_compatible_version, npm_install};
```

To:
```rust
pub mod config;
pub mod packages;

pub use config::{VenConfig, RuntimeConfig, find_ven_toml, parse_ven_toml, load_config, version_spec_resolver};
pub use packages::{fetch_npm_info, find_compatible_version, npm_install};
```

#### 3. Fix `src/core/mod.rs` to remove old functions

Since `config.rs` already has these functions, remove duplicates from `mod.rs`:
- Remove `find_ven_toml` (lines 30-35)
- Remove `parse_ven_toml` (lines 41-49)
- Remove `load_config` (lines 56-61)
- Remove `resolve_node_version` (lines 67-94)
- Remove helper functions `is_lts_version` and `version_cmp` (lines 98-112)

These should stay in `config.rs` where they're already tested.

#### 4. Update `src/cli/mod.rs` imports

Line 5 currently uses:
```rust
use crate::core::{load_config};
```

This should still work after the fix since we're re-exporting from `mod.rs`.

Line 321:
```rust
use crate::core::{load_config, packages::*};
```

Should continue working.

#### 5. Update `src/shell/mod.rs` imports

Line 4:
```rust
use crate::core::{find_ven_toml, parse_ven_toml, resolve_node_version};
```

⚠️ **Problem:** `resolve_node_version` is defined in `mod.rs` but we're removing it.

**Solution:** Move `resolve_node_version` and its helpers to `config.rs`, then re-export.

Add to `config.rs` (after line 63):
```rust
/// Resolve version alias to concrete version string
pub fn resolve_node_version(spec: &str, installed: &[String]) -> Result<String> {
    match spec {
        "latest" => {
            installed.iter()
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Node versions installed. Run: ven install node latest"))
        }
        "lts" => {
            // LTS = even major version numbers (18, 20, 22...)
            installed.iter()
                .filter(|v| is_lts_version(v))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No LTS Node versions installed."))
        }
        spec if !spec.contains('.') => {
            // Major only: "20" → find highest 20.x.x installed
            let major = spec;
            installed.iter()
                .filter(|v| v.starts_with(&format!("{}.", major)))
                .max_by(|a, b| version_cmp(a, b))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No Node {} versions installed.", major))
        }
        _ => Ok(spec.to_string()), // already exact: "20.11.0"
    }
}

fn is_lts_version(version: &str) -> bool {
    // LTS versions have even major numbers: 18.x, 20.x, 22.x
    version.split('.').next()
        .and_then(|major| major.parse::<u32>().ok())
        .map(|n| n % 2 == 0)
        .unwrap_or(false)
}

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    // Compare "20.11.0" vs "22.3.0" numerically
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|n| n.parse().ok()).collect()
    };
    parse(a).cmp(&parse(b))
}
```

Then update `mod.rs` exports to include it.

---

## 📋 Complete Fix Checklist

Before running tests, verify:

- [ ] **Duplicate structs removed** from `mod.rs`
- [ ] **Functions consolidated** in `config.rs`
- [ ] **Re-exports updated** in `mod.rs`
- [ ] **All imports fixed** in:
  - [ ] `cli/mod.rs`
  - [ ] `shell/mod.rs`
  - [ ] `plugins/node.rs`
- [ ] **Tests still pass** (especially `config_test.rs`)
- [ ] **cargo build succeeds** with 0 errors
- [ ] **cargo clippy** shows no warnings

---

## 🧪 Test After Fix

Run these commands to verify the fix worked:

```bash
# Should compile without errors
cargo build --release

# Unit tests should pass
cargo test --all

# No warnings
cargo clippy -- -D warnings
```

---

## 🎯 Why This Happened

This is a common issue in Rust projects during development:

1. Started with simple structs in `mod.rs`
2. Moved to dedicated `config.rs` for better organization
3. Forgot to remove original definitions from `mod.rs`
4. Now both exist → compilation error

**Lesson:** When refactoring Rust code:
- Always check for duplicate definitions
- Use `cargo build` frequently to catch errors early
- Consider using `cargo clippy` which catches these issues

---

## 📞 Need Help?

If you encounter other compilation errors after this fix:

1. Run `cargo build` and read error messages carefully
2. Look for "duplicate definition" or "conflicting implementation"
3. Check module imports/exports are consistent
4. Use `cargo expand` to see macro-expanded code if needed

**Next step:** Apply this fix, then run the test scripts!
