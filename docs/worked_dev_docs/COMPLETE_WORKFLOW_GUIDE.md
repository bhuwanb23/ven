# Complete ven Workflow Guide

**How Every Command Works - Full Technical Breakdown**

---

## 📋 Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Command Workflows](#command-workflows)
3. [Data Flow Diagrams](#data-flow-diagrams)
4. [Key Components](#key-components)

---

## 🏗️ Architecture Overview

### Project Structure
```
ven/
├── main.rs              # Entry point - parses CLI args
├── lib.rs               # Module exports
├── cli/mod.rs           # All CLI command implementations
├── core/
│   ├── config.rs        # ven.toml parsing & management
│   └── packages.rs      # NPM registry API & package logic
├── plugins/
│   ├── mod.rs           # LanguagePlugin trait
│   └── node.rs          # Node.js implementation (uses fnm)
└── shell/
    └── mod.rs           # Shell hook generation & activation
```

### High-Level Flow
```
User runs: ven <command>
    ↓
main.rs: Parse CLI arguments with clap
    ↓
cli/mod.rs: Route to cmd_* function
    ↓
Calls core/ or plugins/ functions
    ↓
Executes external tools (fnm, npm) or API calls
    ↓
Returns Result<()> with success or error message
```

---

## 🎯 Command Workflows

### 1. `ven init [--node <version>]`

**Purpose:** Create new ven.toml configuration file

**Workflow:**
```
1. Get current directory (cwd)
2. Check if ven.toml already exists
   ├─ YES → Error: "ven.toml already exists"
   └─ NO  → Continue

3. Build ven.toml content:
   ├─ If --node flag provided:
   │   └─ Use specified version: node = "18.0.0"
   └─ If no --node flag:
       ├─ Run: node --version
       ├─ Detect current Node version (e.g., "v20.11.0" → "20.11.0")
       └─ Use detected version OR "latest" if detection fails

4. Write ven.toml:
   [runtime]
   node = "20.11.0"
   
   [packages]
   # Add your dependencies here
   # express = "^4.18.2"

5. Success message with next steps
```

**Files Modified:** Creates `ven.toml` in current directory

**External Dependencies:** `node` (optional, for auto-detection)

---

### 2. `ven status`

**Purpose:** Show current project configuration

**Workflow:**
```
1. Get current directory (cwd)
2. Call load_config(&cwd)
   ├─ find_ven_toml(cwd): Walk up directory tree
   │   ├─ Check: cwd/ven.toml exists?
   │   ├─ If no: check parent/ven.toml
   │   └─ Repeat until found or root reached
   └─ parse_ven_toml(path): Read and parse TOML
       ├─ Read file content
       └─ Deserialize to VenConfig struct

3. Display results:
   ├─ If NO config found:
   │   └─ Print: "No ven.toml found. Run: ven init"
   └─ If config found:
       ├─ Print: cwd path
       ├─ Print: node version (from config.runtime.node)
       └─ Print: package count (from config.packages.len())
```

**Files Read:** Searches for `ven.toml` in current directory and parents

**Output Example:**
```
ven status /home/user/my-project
  node 20.11.1
  packages 2 packages declared
```

---

### 3. `ven install node <version>`

**Purpose:** Install a specific Node.js version using fnm backend

**Workflow:**
```
1. Parse language argument (must be "node")
   ├─ If not "node" → Error: "Unknown language"
   └─ If "node" → Continue

2. Resolve version alias:
   ├─ If version == "lts" or "latest":
   │   └─ Call plugin.latest_version()
   │       ├─ Run: fnm list-remote --lts
   │       └─ Parse output to get latest LTS version number
   └─ Otherwise:
       └─ Use version as-is (e.g., "20.11.0")

3. Install via plugin.install_version(resolved_version):
   ├─ Run: fnm install <version>
   ├─ Wait for completion
   └─ Check exit status
       ├─ Success → Print: "✓ Node installed"
       └─ Failure → Error: "fnm failed to install Node X"

4. Done - Node version now available in ~/.fnm/node-versions/
```

**Files Modified:** Downloads Node.js to `~/.fnm/node-versions/v<version>/`

**External Dependencies:** Requires `fnm` (Fast Node Manager)

**Example:**
```bash
ven install node 20.11.0
# Runs: fnm install 20.11.0
# Downloads to: ~/.fnm/node-versions/v20.11.0/installation/
```

---

### 4. `ven list [node]`

**Purpose:** List all installed Node.js versions

**Workflow:**
```
1. Parse language argument (defaults to "node")
2. Call plugin.list_installed():
   ├─ Run: fnm list
   ├─ Capture stdout:
   │   * v20.11.0 default
   │   v22.3.0
   │   v18.20.0
   └─ Parse each line:
       ├─ Remove '*' marker (current version indicator)
       ├─ Remove 'v' prefix from version
       └─ Extract version number only

3. Display results:
   ├─ If no versions found:
   │   └─ Warning: "No Node versions installed. Run: ven install node latest"
   └─ If versions found:
       └─ Print formatted list with bullet points
```

**External Dependencies:** Requires `fnm`

**Output Example:**
```
  node
    • 20.11.0
    • 22.3.0
    • 18.20.0
```

---

### 5. `ven add <package> [--skip-check]`

**Purpose:** Add package with compatibility checking

**Workflow:**
```
1. Parse package spec:
   ├─ If "express@4.18.2":
   │   ├─ pkg_name = "express"
   │   └─ pinned_version = "4.18.2"
   └─ If "express":
       ├─ pkg_name = "express"
       └─ pinned_version = None

2. Load Node version from ven.toml:
   └─ load_config(cwd).runtime.node

3. Check --skip-check flag:
   ├─ IF TRUE (skip compatibility check):
   │   ├─ Print: "Skipping compatibility check..."
   │   ├─ Call npm_install(pkg_name, "latest")
   │   │   └─ Run: npm install lodash@latest
   │   ├─ Update ven.toml with "latest"
   │   └─ Return early (no API call!)
   │
   └─ IF FALSE (normal path):
       ├─ Print: "Checking <pkg> against Node X..."
       ├─ Call fetch_npm_info(pkg_name)
       │   ├─ GET https://registry.npmjs.org/<package>
       │   ├─ Parse JSON response to NpmPackageInfo
       │   └─ Handle errors (404, parse failures, network issues)
       └─ Call find_compatible_version(info, node_version)
           ├─ Check "latest" tag first
           │   └─ Verify engine requirements match Node version
           ├─ If not compatible, sort all versions descending
           └─ Find highest version where engines.node is satisfied

4. Determine version_to_install:
   ├─ User specified exact version → Use it
   ├─ skip_check=true → Use "latest"
   └─ Normal path → Use compatible version found above

5. Install package:
   └─ npm_install(pkg_name, version_to_install)
       └─ Run: npm install express@4.18.2

6. Update ven.toml:
   └─ Append: express = "4.18.2" to [packages] section

7. Success messages
```

**Files Modified:** 
- Installs to `node_modules/`
- Updates `ven.toml`
- Updates `package-lock.json` (via npm)

**External Dependencies:** Requires `npm`, needs internet for registry API

**API Flow:**
```
ven add express
    ↓
GET https://registry.npmjs.org/express
    ↓
Response JSON:
{
  "name": "express",
  "dist-tags": {"latest": "4.18.2"},
  "versions": {
    "4.18.2": {"engines": {"node": ">= 0.10.0"}},
    ...
  }
}
    ↓
Check: Does "4.18.2" satisfy Node "20.11.0"?
    ↓
YES → Install express@4.18.2
```

---

### 6. `ven remove <package> [--force]`

**Purpose:** Remove package with dependency checking

**Workflow:**
```
1. Parse package name and --force flag

2. If NOT --force:
   └─ Call find_dependents(package):
       ├─ Read package-lock.json
       ├─ Parse "packages" section
       ├─ For each package, check its "dependencies"
       └─ Collect list of packages that depend on target

3. If dependents found AND NOT --force:
   ├─ Print warning with list of dependent packages
   ├─ Ask: "Remove anyway? [y/N]:"
   ├─ Read user input
   └─ If not "y" → Cancel and return
       
4. Uninstall:
   └─ npm_uninstall(package)
       └─ Run: npm uninstall express

5. Note: ven.toml NOT automatically updated (TODO)

6. Success message
```

**Files Modified:** 
- Removes from `node_modules/`
- Updates `package-lock.json`

**External Dependencies:** Requires `npm`

---

### 7. `ven upgrade <package> [--apply]`

**Purpose:** Preview or apply package upgrade

**Workflow:**
```
1. Load Node version from ven.toml

2. Get currently installed version:
   └─ get_installed_version(package):
       ├─ Read node_modules/<package>/package.json
       └─ Extract "version" field

3. Fetch latest compatible version:
   ├─ fetch_npm_info(package)
   └─ find_compatible_version(info, node_version)

4. Compare versions:
   ├─ If current == latest:
   │   └─ Print: "Already up to date"
   │   └─ Return
   └─ If different:
       └─ Continue

5. Preview mode (default, --apply NOT specified):
   ├─ Print: "package  current → latest (compatible)"
   ├─ Print: "Compatibility: ✓ Node X supported"
   ├─ Print: "Release notes: <changelog link>"
   └─ Print: "Run: ven upgrade package --apply to upgrade"

6. Apply mode (--apply specified):
   ├─ npm_install(package, latest_version)
   ├─ update_ven_toml_package(package, latest_version)
   └─ Print: "✓ Upgraded to latest"
```

**Files Modified (with --apply):**
- Updates `node_modules/<package>`
- Updates `ven.toml`

**External Dependencies:** Requires `npm`, needs internet for registry API

---

### 8. `ven setup`

**Purpose:** Install shell hook for automatic activation

**Workflow:**
```
1. Detect current shell:
   ├─ Read $SHELL environment variable
   ├─ Extract shell name (e.g., "/bin/bash" → "bash")
   └─ Default to "bash" if unknown

2. Find shell rc file:
   ├─ bash/zsh → ~/.bashrc
   ├─ fish → ~/.config/fish/config.fish
   └─ zsh → ~/.zshrc

3. Check if already installed:
   ├─ Read rc file content
   └─ Search for "# ven shell hook" comment
       ├─ Found → Print: "Already installed"
       └─ Not found → Continue

4. Append hook code:
   └─ Write to rc file:
       # ven shell hook
       eval "$(ven shell <shell_name>)"

5. Instructions:
   └─ Print: "Restart shell or run: source ~/.bashrc"
```

**Files Modified:** Appends to `~/.bashrc` or equivalent

**External Dependencies:** None

---

### 9. `ven shell hook <shell>`

**Purpose:** Generate shell hook code

**Workflow:**
```
1. Parse shell argument (bash, zsh, fish)

2. Generate hook based on shell type:
   ├─ bash/zsh:
   │   └─ Return bash function code:
   │       __ven_activate() {
   │           exports=$(ven shell activate "$PWD")
   │           eval "$exports"
   │       }
   │       cd() { builtin cd "$@" && __ven_activate; }
   │
   └─ fish:
       └─ Return fish function code:
           function __ven_activate --on-variable PWD
               set exports (ven shell activate "$PWD")
               eval $exports
           end

3. Print hook code to stdout
```

**Output:** Shell script code (not executed, just printed)

**Used By:** Called via `eval "$(ven shell hook bash)"` in shell rc file

---

### 10. `ven shell activate <directory>`

**Purpose:** Compute PATH exports for a specific directory

**Workflow:**
```
1. Parse directory path argument

2. Find ven.toml:
   └─ find_ven_toml(dir):
       ├─ Start at dir
       ├─ Check: dir/ven.toml exists?
       ├─ If no: check parent/ven.toml
       └─ Repeat until found or root reached
           ├─ Found → Return path
           └─ Not found → Return None

3. If NO ven.toml found:
   └─ Return None (print nothing)

4. If ven.toml found:
   ├─ parse_ven_toml(path) → VenConfig struct
   ├─ Extract: config.runtime.node (e.g., "20.11.1")
   
5. Resolve version alias:
   └─ resolve_node_version(spec, installed_versions):
       ├─ If "latest" → Find highest installed
       ├─ If "lts" → Find highest even major version
       ├─ If "20" → Find highest 20.x.x
       └─ Otherwise → Use exact version

6. Get bin path:
   └─ plugin.bin_path(resolved_version):
       └─ Return: ~/.fnm/node-versions/v20.11.1/installation/bin

7. Build export string:
   └─ Format:
       export PATH="/home/user/.fnm/node-versions/v20.11.1/installation/bin:$PATH"
       export VEN_NODE_VERSION="20.11.1"
       export VEN_TOML="/home/user/project/ven.toml"

8. Add environment variables from [env] section:
   └─ If config.env not empty:
       export NODE_ENV="development"
       export PORT="3000"

9. Print exports to stdout
```

**Output:** Shell export commands (meant to be eval'd)

**Used By:** Called internally by shell hook on every `cd`

---

## 🔄 Data Flow Diagrams

### Configuration Loading Flow
```
Any command needing config
    ↓
load_config(current_dir)
    ↓
find_ven_toml(current_dir)
    ├─ Check: current_dir/ven.toml
    ├─ Not found? Check: parent_dir/ven.toml
    ├─ Not found? Check: grandparent/ven.toml
    └─ Continue until root or found
    ↓
Found ven.toml at /path/to/ven.toml
    ↓
parse_ven_toml(path)
    ├─ Read file to string
    ├─ Parse TOML with serde
    └─ Return VenConfig struct
    ↓
Use config.runtime.node, config.packages, etc.
```

### Package Installation Flow
```
ven add express
    ↓
Load config → Get Node version (e.g., "20.11.0")
    ↓
fetch_npm_info("express")
    ├─ HTTP GET: https://registry.npmjs.org/express
    ├─ Receive JSON response
    ├─ Parse to NpmPackageInfo struct
    └─ Extract: dist-tags, versions, engines
    ↓
find_compatible_version(info, "20.11.0")
    ├─ Check "latest" tag version
    ├─ Read: versions["4.18.2"].engines.node
    ├─ Check: Does "20.11.0" satisfy ">= 0.10.0"?
    └─ YES → Return "4.18.2"
    ↓
npm_install("express", "4.18.2")
    └─ Run: npm install express@4.18.2
    ↓
update_ven_toml_package("express", "4.18.2")
    └─ Append: express = "4.18.2"
```

### Shell Activation Flow
```
User: cd my-project/
    ↓
Shell hook triggers: __ven_activate()
    ↓
Call: ven shell activate "$PWD"
    ↓
Find ven.toml in my-project/
    ↓
Parse config → node = "20.11.1"
    ↓
Resolve alias → "20.11.1" (already resolved)
    ↓
Get bin path: ~/.fnm/node-versions/v20.11.1/installation/bin
    ↓
Generate exports:
    export PATH="<bin>:$PATH"
    export VEN_NODE_VERSION="20.11.1"
    export VEN_TOML="<path>"
    ↓
eval "$exports" in current shell
    ↓
Node 20.11.1 now active!
```

---

## 🧩 Key Components

### 1. VenConfig Struct
```rust
pub struct VenConfig {
    pub runtime: RuntimeConfig,      // Node version
    pub packages: HashMap<String, String>,  // Dependencies
    pub env: HashMap<String, String>,       // Environment vars
}

pub struct RuntimeConfig {
    pub node: String,  // e.g., "20.11.1"
}
```

**Location:** `src/core/config.rs`

**Purpose:** Represents entire ven.toml configuration

---

### 2. LanguagePlugin Trait
```rust
pub trait LanguagePlugin {
    fn name(&self) -> &str;
    fn install_version(&self, version: &str) -> Result<()>;
    fn list_installed(&self) -> Result<Vec<String>>;
    fn bin_path(&self, version: &str) -> Result<PathBuf>;
    fn latest_version(&self) -> Result<String>;
}
```

**Location:** `src/plugins/mod.rs`

**Implementations:**
- `NodePlugin` (in `node.rs`) - delegates to fnm

**Purpose:** Standard interface for language support

---

### 3. NPM Registry Integration
```rust
pub struct NpmPackageInfo {
    pub name: String,
    pub dist_tags: HashMap<String, String>,  // {"latest": "4.18.2"}
    pub versions: HashMap<String, NpmVersionInfo>,
}

pub struct NpmVersionInfo {
    pub engines: Option<HashMap<String, String>>,  // {"node": ">= 0.10.0"}
}
```

**Location:** `src/core/packages.rs`

**Purpose:** Parse npm registry JSON responses

---

### 4. Shell Hook System

**Two Parts:**

**Part 1: Hook Generator** (`generate_hook`)
- Generates shell-specific function code
- Overrides `cd` command to trigger activation
- Supports: bash, zsh, fish

**Part 2: Export Computer** (`compute_exports`)
- Reads ven.toml for current directory
- Computes correct PATH for Node version
- Generates export commands for shell to eval

**Integration:**
```bash
# In ~/.bashrc:
eval "$(ven shell hook bash)"

# On every cd:
__ven_activate() {
    exports=$(ven shell activate "$PWD")
    eval "$exports"  # ← Sets PATH, VEN_NODE_VERSION, etc.
}
```

---

## 🎯 Summary: How It All Works Together

### The Big Picture

1. **Configuration** (`ven.toml`)
   - Single source of truth for project
   - Specifies Node version + packages
   - Auto-discovered by walking up directory tree

2. **Version Management** (delegates to fnm)
   - Downloads Node.js versions to isolated folders
   - Swaps PATH to activate correct version
   - Tracks what's installed

3. **Package Management** (delegates to npm)
   - Checks compatibility BEFORE install
   - Queries npm registry API
   - Updates ven.toml automatically

4. **Shell Integration**
   - Hooks into `cd` command
   - Auto-activates correct Node version per project
   - Exports environment variables

5. **Plugin Architecture**
   - Node.js implemented via fnm
   - Ready for Python, Go, Ruby, Rust
   - Same interface for all languages

---

**This is your complete ven workflow reference!** Every command, data flow, and component explained. 🎉
