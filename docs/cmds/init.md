# ven init

Create a `ven.toml` configuration file with interactive templates and package selection.

## Overview

The `init` command is ven's **project initialization wizard** that helps you set up a new project with the right configuration.

Unlike manually writing `ven.toml`, `ven init`:

- ✅ Interactive template selection
- ✅ Smart language/version detection
- ✅ Multi-select package picker
- ✅ Post-creation validation
- ✅ Best practice recommendations

> **Scope in this release (v0.1.x):**
> The wizard pins **one language at a time**. The `ven.toml` *schema* and every other ven command (`add`, `status`, the activation hook, `lock`, `sync`, `check`, …) already support **multiple runtimes per project** — you just declare them by editing `ven.toml` directly after `ven init`. See [Multi-runtime projects](#multi-runtime-projects) below for the exact pattern.
>
> A multi-select wizard (`SPACE` to toggle several languages, then a version prompt for each) is on the roadmap for the next release.

## Usage

### Basic Initialization

```bash
ven init [OPTIONS]
```

### Examples

#### Interactive Template Mode

```bash
ven init --template
```

**Process:**
```
📦 Smart Project Templates

? Select project template:
  ▸ Express API Server
    React + Vite Frontend
    Next.js Full-stack
    Empty Project
```

**After selecting "Express API Server":**
```
✓ Selected: Express API Server
  🔧 node 20

📦 Created ven.toml
  📦 Template: Express API Server
  🔧 node 20
  📦 3 packages:
      express ^4.18.2
      cors ^2.8.5
      dotenv ^16.3.1
```

#### Interactive Package Selection

```bash
ven init --with-packages
```

**Process:**
1. Select language (node, python)
2. Select version
3. Multi-select popular packages

**Output:**
```
? Select language:
  ▸ node
    python

? Select Node.js version:
  ▸ 20.11.0  ⭐ LTS     (~98% pkg compat) [Recommended]
    22.3.0   ✅ Current  (~95% pkg compat)
    lts                 Latest LTS (recommended)
    20                  Active LTS (best compatibility)

📦 Interactive Package Selection

? Add popular packages? (SPACE to select, ENTER to continue)
  ❯ ◉ express ^4.18.2 - Fast, minimalist web framework
    ◉ typescript ^5.3.0 - Typed JavaScript for better DX
    ◉ dotenv ^16.3.1 - Load environment variables from .env
    ◉ cors ^2.8.5 - Enable CORS support
    ◉ morgan ^1.10.0 - HTTP request logger
    ◉ jest ^29.7.0 - Testing framework
    ◉ eslint ^8.56.0 - Code linting and quality
    ◉ prettier ^3.1.0 - Code formatting
```

#### Validation Mode

```bash
ven init --template --validate
```

**Output:**
```
📦 Created ven.toml
  📦 Template: Express API Server
  🔧 node 20
  📦 3 packages:
      express ^4.18.2
      cors ^2.8.5
      dotenv ^16.3.1

🔍 Running validation...
  ✓ ven.toml created
  ⚠️ Node.js 20 (will resolve during install)
  ✓ 3 packages declared
      • express
      • cors
      • dotenv
  ℹ️ Environment variables (optional)

🚀 Ready to develop!
```

#### Legacy Mode (Quick)

```bash
ven init --node 20.11.0
```

**Output:**
```
✓ Created ven.toml
  🔧 node 20.11.0

Edit this file to customize your dependencies.
Run: ven install node 20.11.0   to install this version
```

---

## Command Reference

### Syntax

```bash
ven init [OPTIONS]
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--template` | Use interactive template selection | `false` |
| `--with-packages` | Add popular packages interactively | `false` |
| `--validate` | Validate setup after creation | `false` |
| `--node <version>` | Specify Node version (legacy) | Auto-detect |

---

## Templates

### 1. Express API Server

**Purpose:** Backend REST API

**Configuration:**
```toml
[runtime]
node = "20"

[packages]
express = "^4.18.2"
cors = "^2.8.5"
dotenv = "^16.3.1"
```

**Includes:**
- **express**: Web framework
- **cors**: Cross-origin support
- **dotenv**: Environment variables

**Best for:**
- REST APIs
- Microservices
- Backend servers

---

### 2. React + Vite Frontend

**Purpose:** Modern frontend application

**Configuration:**
```toml
[runtime]
node = "20"

[packages]
react = "^18.2.0"
react-dom = "^18.2.0"
vite = "^5.0.0"
```

**Includes:**
- **react**: UI library
- **react-dom**: React DOM renderer
- **vite**: Fast build tool

**Best for:**
- Single-page apps
- Modern frontend
- Fast development

---

### 3. Next.js Full-stack

**Purpose:** Full-stack React application

**Configuration:**
```toml
[runtime]
node = "20"

[packages]
next = "^14.0.0"
react = "^18.2.0"
react-dom = "^18.2.0"
```

**Includes:**
- **next**: React framework
- **react**: UI library
- **react-dom**: DOM renderer

**Best for:**
- SSR applications
- Full-stack apps
- SEO-friendly sites

---

### 4. Empty Project

**Purpose:** Start from scratch

**Process:**
1. Select language (node, python)
2. Select version interactively
3. Optionally add packages

**Configuration:**
```toml
[runtime]
node = "20"

[packages]
# Add your dependencies here
# express = "^4.18.2"
```

**Best for:**
- Custom setups
- Learning projects
- Experimental code

---

## Language Selection

### Node.js

**Version Options:**
```
? Select Node.js version:
  ▸ 20.11.0  ⭐ LTS     (~98% pkg compat) [Recommended]
    22.3.0   ✅ Current  (~95% pkg compat)
    lts                 Latest LTS (recommended)
    22                  Current release line
    20                  Active LTS (best compatibility)
    18                  Maintenance LTS
```

**Version Metadata:**
- **LTS** ⭐: Long Term Support, highest compatibility
- **Current**: Latest features, good compatibility
- **Aliases**: `lts`, `latest`, major versions

### Python

```
? Select Python version:
  ▸ 3.13.12  ⭐ Stable
    3.12.7
    3.11.8
    3.10.14
    latest
```

When Python is the picked language, `ven init` additionally:

1. Writes a `[venv]` block with `auto_path = true` so the activation hook prepends `./venv/bin` (or `./.venv/bin`) to `PATH`.
2. Creates `./venv` via `python -m venv --copies`. If the embeddable build lacks the stdlib `venv` module (common on Windows), it `pip install`s `virtualenv` and uses that instead.
3. Adds `venv/` and `.venv/` to `.gitignore` if a git repo is detected.

The remaining six runtimes (`go`, `rust`, `java`, `deno`, `bun`, `ruby`) work the same way — interactive picker, version prompt, single line written to `[runtime]`. Pick the rest by adding them to `ven.toml` directly today, or wait for the multi-select wizard in the next release.

---

## Package Selection

### Popular Packages

Multi-select picker includes:

| Package | Version | Description |
|---------|---------|-------------|
| express | ^4.18.2 | Fast, minimalist web framework |
| typescript | ^5.3.0 | Typed JavaScript for better DX |
| dotenv | ^16.3.1 | Load environment variables from .env |
| cors | ^2.8.5 | Enable CORS support |
| morgan | ^1.10.0 | HTTP request logger |
| jest | ^29.7.0 | Testing framework |
| eslint | ^8.56.0 | Code linting and quality |
| prettier | ^3.1.0 | Code formatting |

### Selection Interface

```
? Add popular packages? (SPACE to select, ENTER to continue)

Use arrow keys to navigate, SPACE to select:

  ❯ ◉ express ^4.18.2 - Fast, minimalist web framework
    ◉ typescript ^5.3.0 - Typed JavaScript for better DX
    ◉ dotenv ^16.3.1 - Load environment variables from .env
    ◉ cors ^2.8.5 - Enable CORS support
    ◉ morgan ^1.10.0 - HTTP request logger
    ◉ jest ^29.7.0 - Testing framework
    ◉ eslint ^8.56.0 - Code linting and quality
    ◉ prettier ^3.1.0 - Code formatting
```

**Controls:**
- `↑/↓`: Navigate
- `SPACE`: Select/deselect
- `ENTER`: Confirm

---

## Validation System

### What It Checks

1. **ven.toml created**: File exists and is valid
2. **Node.js version installed**: Checks if version is available
3. **Package compatibility**: Lists declared packages
4. **Environment variables**: Notes optional env vars

### Output Format

```
🔍 Running validation...
  ✓ ven.toml created
  ⚠️ Node.js 20 (will resolve during install)
  ✓ 3 packages declared
      • express
      • cors
      • dotenv
  ℹ️ Environment variables (optional)

🚀 Ready to develop!
```

### Status Icons

| Icon | Meaning |
|------|---------|
| ✓ | Check passed |
| ⚠️ | Warning (needs attention) |
| ✗ | Error (must fix) |
| ℹ️ | Information |
| 💡 | Tip |

---

## Generated ven.toml

### Structure

```toml
[runtime]
node = "20"

[packages]
express = "^4.18.2"
cors = "^2.8.5"
dotenv = "^16.3.1"
```

### Sections

#### [runtime]

**Purpose:** Pin one or more language versions for this project. Every field is independent — populate as many as you want; the activation hook and every other ven command will see them all.

**Fields:** all eight are accepted by the schema today.
- `node` — Node.js
- `python` — CPython
- `go` — Go toolchain
- `rust` — Rust toolchain (via rustup)
- `java` — JDK
- `deno` — Deno
- `bun` — Bun
- `ruby` — MRI Ruby

> The `ven init` wizard currently writes **one** of these per run (whichever you pick at the language prompt). Add the rest by editing `ven.toml` directly — see [Multi-runtime projects](#multi-runtime-projects).

**Single-runtime example:**
```toml
[runtime]
node = "20.11.0"  # Exact version
node = "20"       # Major version
node = "lts"      # Alias
```

**Multi-runtime example** (hand-written today, wizard-supported in the next release):
```toml
[runtime]
node = "20"
python = "3.12"
go = "1.22"
```

#### [packages]

**Purpose:** Track project dependencies

**Format:** `package = "version"`

**Example:**
```toml
[packages]
express = "^4.18.2"
react = "18.2.0"
typescript = "^5.3.0"
```

**Note:** ven.toml tracks your **intended** packages. Actual installation is in `node_modules`.

#### [env] (Optional)

**Purpose:** Define environment variables

**Example:**
```toml
[env]
NODE_ENV = "development"
PORT = "3000"
API_URL = "http://localhost:8080"
```

**Usage:** Variables are automatically set when you `cd` into the directory (via shell hook).

---

## Multi-runtime projects

`ven` is designed for projects that span more than one language — a Python service with a Node-built frontend, a Go API with Python data scripts, a Rust crate with a docs site on Bun. The `ven.toml` schema treats every runtime field as independent, so you can pin two or eight in the same project and the activation hook will set up `PATH`, `JAVA_HOME`, `GEM_HOME`, `CARGO_HOME`, etc. **all at once** when you `cd` in.

### How to do it today

1. Run `ven init` (or `ven init --template`) and pick whichever language is the *primary* one for the project. The wizard writes a single-runtime `ven.toml`.
2. Open the file in your editor.
3. Add additional runtime lines directly under `[runtime]`. Each one accepts the same version-spec format as the wizard (exact, major, alias):

   ```toml
   [runtime]
   node = "20"        # written by `ven init`
   python = "3.12"    # added by hand
   go = "1.22"        # added by hand
   ```

4. Install the new runtimes:

   ```bash
   ven install python 3.12
   ven install go 1.22
   ```

5. `cd` out and back in (or run `ven status`) — the hook now activates all three.

### What works for multi-runtime projects today

| Command | Behavior with multiple `[runtime]` entries |
|---------|---------------------------------------------|
| `ven status`         | Reports every populated runtime, side-by-side |
| `ven add <pkg>`      | Routes to the right package manager based on the package's ecosystem hints (npm, pip, cargo, go, gem, …) |
| `ven remove <pkg>`   | Same |
| `ven check`          | Scans **every** declared runtime against OSV / EOL |
| `ven scan --ghosts`  | Walks every source file, attributes ghosts to the right runtime |
| `ven lock` / `ven sync` | One `ven.lock` covers every runtime's package set |
| Shell hook           | Sets PATH + per-language env vars for **all** declared runtimes simultaneously |

### What's coming in the next release

- `ven init` multi-select prompt — pick `[ ] node`, `[ ] python`, … with `SPACE`, then a per-language version prompt.
- A `--lang node,python,go` headless syntax for CI scaffolding.
- Multi-language templates (e.g. "Django + Vite frontend").

---

## Use Cases

### 1. New Project (single language)

```bash
mkdir myproject && cd myproject
ven init --template
ven setup
ven install node 20
ven add express
```

### 1b. New Project (multi-runtime, hand-edit step)

```bash
mkdir polyglot-app && cd polyglot-app
ven init --lang python --ver 3.12

# Open ven.toml and add a second runtime by hand:
#   [runtime]
#   python = "3.12"
#   node   = "20"          ← add this line

ven install node 20
cd . && ven status      # both runtimes active
```

### 2. Monorepo Setup

```bash
# Frontend
mkdir monorepo/frontend && cd monorepo/frontend
ven init --template  # Select "React + Vite"

# Backend
mkdir ../backend && cd ../backend
ven init --template  # Select "Express API Server"
```

### 3. Existing Project

```bash
cd existing-project/
ven init --with-packages  # Add packages interactively
```

### 4. Quick Start

```bash
mkdir api && cd api
ven init --node 20
ven setup
ven add express
```

---

## Implementation Details

### Source Code Location

- **CLI handler**: [`src/cli/init.rs`](../../src/cli/init.rs) (360 lines)

### Key Functions

```rust
// Main entry point
cmd_init(node, use_template, with_packages, validate)

// Interactive selection
select_node_version()
select_python_version()

// Validation
run_validation(language, version, packages)

// Metadata
get_version_info(version)
```

### Dependencies

```toml
dialoguer 0.11    # Interactive prompts
colored 2         # Terminal colors
anyhow 1          # Error handling
```

---

## Error Handling

### ven.toml Already Exists

```bash
ven init
```

**Output:**
```
Error: ven.toml already exists in this directory
```

**Solution:**
```bash
# Edit existing file
nano ven.toml

# Or remove and recreate
rm ven.toml
ven init
```

### Invalid Selection

```bash
# During template selection
? Select project template:
# Press Enter on separator
```

**Output:**
```
Error: Please select a valid version
```

**Solution:** Select a valid option (not a separator).

---

## Best Practices

### 1. Use Templates

```bash
# Start with a template
ven init --template

# Customize after creation
nano ven.toml
```

### 2. Validate Setup

```bash
# Always validate
ven init --template --validate

# Fix any issues before starting development
```

### 3. Add Packages Incrementally

```bash
# Start minimal
ven init --template

# Add as needed
ven add typescript
ven add eslint
ven add prettier
```

### 4. Version Pinning

```toml
# Pin exact versions for production
[packages]
express = "4.18.2"

# Or use semver ranges for development
express = "^4.18.2"
```

---

## Troubleshooting

### Wrong Version Selected

**Problem:** Selected wrong version during init

**Solution:**
```bash
# Edit ven.toml
nano ven.toml

# Change node version
# node = "20"  →  node = "18"

# Install correct version
ven install node 18
```

### Template Not Available

**Problem:** Want template that doesn't exist

**Solution:**
```bash
# Start with empty project
ven init --template  # Select "Empty Project"

# Manually add packages
ven add your-package
```

### Validation Fails

**Problem:** `--validate` shows issues

**Solution:**
```bash
# Read warnings carefully
# Install missing versions
ven install node 20

# Re-validate
ven status
```

---

## Related Commands

- [`ven setup`](setup.md) - Configure shell auto-switching
- [`ven add`](add.md) - Add more packages
- [`ven install`](install.md) - Install Node.js versions
- [`ven status`](status.md) - Check project config

---

## Next Steps

After initialization:

```bash
# 1. Install Node.js version
ven install node 20

# 2. Install packages
ven add express
ven add typescript

# 3. Set up auto-switching
ven setup

# 4. Start developing
npm run dev
```

For complete workflow, see [COMPLETE_WORKFLOW_GUIDE.md](../COMPLETE_WORKFLOW_GUIDE.md).
