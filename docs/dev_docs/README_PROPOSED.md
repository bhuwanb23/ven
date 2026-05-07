# ven - Predictive Dependency Manager

**Status:** Phase 1 Complete ✅ | Phase 2 Planning 🔄  
**Language:** Rust  
**Target:** Multi-language (Node.js implemented, Python/Go/Ruby planned)

---

## 🎯 What is ven?

ven is a **predictive** language version and dependency manager that answers the critical question before any installation:

> **"Can I safely add this package to my current stack?"**

Unlike reactive tools (npm, nvm, pip) that install first and report failures second, ven builds a complete compatibility graph and walks your dependency tree **before** a single byte is downloaded.

---

## ✨ Key Features

### Phase 1 (Complete ✅)
- **Predictive Compatibility Checking** — Know if a package works with your Node version before installing
- **Per-Project Version Isolation** — Automatic Node version switching on `cd` via PATH manipulation
- **Shell Integration** — Works with bash, zsh, and fish
- **Smart Package Management** — `ven add` checks engine requirements, `ven remove` warns about dependents
- **Deterministic Configs** — `ven.toml` for reproducible environments

### Phase 2 (Planned 🔄)
- **Ghost Dependency Detection** — Scan imports not declared in dependencies
- **Vulnerability Monitoring** — OSV database + endoflife.date integration
- **Version-Pinned Documentation** — Docs matching your exact installed versions
- **Lock File Generation** — `ven.lock` for bit-for-bit reproducible installs
- **Multi-Language Support** — Python, Go, Ruby, Rust plugins
- **Export Capabilities** — Generate Dockerfile / GitHub Actions from ven.toml

---

## 🚀 Quick Start

### Installation

**Prerequisites:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install fnm (Fast Node Manager)
curl -fsSL https://fnm.vercel.app/install | bash
```

**Build from source:**
```bash
git clone https://github.com/yourusername/ven.git
cd ven
cargo build --release
```

### First Project

```bash
# Create new project
mkdir my-project && cd my-project

# Initialize ven.toml with current Node version
ven init

# Install a specific Node version
ven install node 20.11.0

# Add a package with compatibility check
ven add express

# Check status
ven status
```

### Shell Integration

```bash
# One-time setup (adds hook to ~/.bashrc or ~/.zshrc)
ven setup

# Restart shell, then automatic activation works:
cd project-with-node-20/   # → Node 20 activates
cd project-with-node-18/   # → Node 18 activates
```

---

## 📖 Configuration

### ven.toml Format

```toml
[runtime]
node = "20.11.1"  # Can use: "latest", "lts", "20", or exact version

[packages]
express = "^4.18.2"
react = "18.2.0"

[env]
NODE_ENV = "production"
PORT = "3000"
```

### Version Specifiers

| Specifier | Resolves To |
|-----------|-------------|
| `"latest"` | Newest Node.js version |
| `"lts"` | Latest LTS (even major: 18, 20, 22) |
| `"20"` | Latest 20.x.x |
| `"20.11.0"` | Exact version |

---

## 🛠️ CLI Commands

```bash
ven init [--node <version>]     # Create ven.toml
ven install node <version>      # Install Node.js version
ven list [node]                 # List installed versions
ven status                      # Show current config
ven add <package>               # Add package with compatibility check
ven remove <package> [--force]  # Remove package (warns about dependents)
ven upgrade <package> [--apply] # Preview or apply upgrade
ven setup                       # Install shell hook
```

---

## 🧪 Testing Status

**Phase 1 Completion:** 95% code complete, awaiting testing

### Test Coverage
- ✅ Unit tests written (6 tests)
- ✅ Integration test scenarios documented
- ⏳ Automated test runners created (bash + PowerShell)
- ⏳ Manual testing pending

### Known Issues
- 🔴 **CRITICAL:** Duplicate struct definitions in `src/core/mod.rs` prevent compilation
  - **Fix Time:** ~30 minutes
  - **Instructions:** See `BUG_FIX_NEEDED.md`

### How to Test

**Quick Start (30 min):**
1. Install Rust toolchain
2. Fix duplicate structs (see `BUG_FIX_NEEDED.md`)
3. `cargo build --release`
4. `cargo test --all`
5. Run manual tests from `TESTING_PLAN.md`

**Full Documentation:**
- 📄 [`QUICK_START_TESTING.md`](./QUICK_START_TESTING.md) — 5-step quick start
- 📄 [`TESTING_PLAN.md`](./TESTING_PLAN.md) — Comprehensive test scenarios
- 📄 [`BUG_FIX_NEEDED.md`](./BUG_FIX_NEEDED.md) — Critical fix instructions
- 📄 [`PHASE1_STATUS.md`](./PHASE1_STATUS.md) — Detailed status report

**Test Scripts:**
- `scripts/test_phase1.sh` — Linux/Mac automated runner
- `scripts/test_phase1.ps1` — Windows automated runner

---

## 🏗️ Architecture

### Plugin System

ven uses a plugin architecture for multi-language support:

```rust
pub trait LanguagePlugin {
    fn name(&self) -> &str;
    fn install_version(&self, version: &str) -> Result<()>;
    fn list_installed(&self) -> Result<Vec<String>>;
    fn bin_path(&self, version: &str) -> Result<PathBuf>;
    fn latest_version(&self) -> Result<String>;
}
```

**Implemented:**
- ✅ Node.js (via fnm backend)

**Planned:**
- 🔄 Python (via pyenv backend)
- 🔄 Go (via goenv backend)
- 🔄 Ruby (via rbenv backend)
- 🔄 Rust (via rustup backend)

### Directory Structure

```
ven/
├── src/
│   ├── cli/          # Command-line interface (clap)
│   ├── core/         # Config parsing, package logic
│   ├── intelligence/ # Phase 2: Compatibility graph
│   ├── health/       # Phase 2: Vulnerability monitoring
│   ├── docs/         # Phase 2: Version-pinned docs
│   ├── sync/         # Phase 2: Lock file generation
│   ├── plugins/      # Language backends
│   └── shell/        # Shell hook generation
├── tests/            # Integration tests
└── example/          # Example project configs
```

---

## 🎯 Design Philosophy

### What ven Is

✅ **An intelligence layer** above existing tools  
✅ **A predictive system** that checks before installing  
✅ **A unifying interface** across languages  
✅ **A team synchronization** tool  

### What ven Is Not

❌ **Not replacing** npm/pip/cargo — delegates to them  
❌ **Not reinventing** version downloaders — uses fnm/pyenv/rustup  
❌ **Not a package registry** — works with existing ecosystems  

---

## 📊 Roadmap

### Phase 1 ✅ (Weeks 0-7) — Foundation
- [x] Core architecture and CLI
- [x] Node.js plugin
- [x] Shell integration
- [x] Basic package management
- [ ] Testing and bug fixes (IN PROGRESS)

### Phase 2 🔄 (Weeks 8-14) — Intelligence
- [ ] Ghost dependency detection
- [ ] Vulnerability monitoring (OSV + endoflife.date)
- [ ] Lock file generation (`ven.lock`)
- [ ] Python plugin
- [ ] Version-pinned documentation

### Phase 3 (Weeks 15-20) — Advanced Features
- [ ] Go, Ruby, Rust plugins
- [ ] Export to Dockerfile / GitHub Actions
- [ ] Team environment sync
- [ ] Registry metadata caching (SQLite)

### Phase 4 (Future) — Ecosystem
- [ ] IDE integrations
- [ ] CI/CD optimizations
- [ ] Enterprise features (private registries, proxies)

---

## 🤝 Contributing

### Current Priorities

**Immediate Needs:**
1. Fix duplicate struct definitions (see `BUG_FIX_NEEDED.md`)
2. Execute comprehensive testing (see `TESTING_PLAN.md`)
3. Document bugs found during testing
4. Polish error messages and UX

**Phase 2 Contributions Welcome In:**
- Ghost dependency scanner (AST parsing)
- OSV vulnerability API integration
- endoflife.date version tracking
- Python plugin implementation

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/ven.git
cd ven

# Build
cargo build --release

# Run tests
cargo test --all

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

---

## 📚 Documentation

### For Users
- This README — Overview and quick start
- `QUICK_START_TESTING.md` — Testing guide
- `TESTING_PLAN.md` — Full test scenarios

### For Developers
- `BUG_FIX_NEEDED.md` — Critical issue documentation
- `PHASE1_STATUS.md` — Detailed progress report
- Source code comments — Inline documentation

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [clap Documentation](https://docs.rs/clap/latest/clap/)
- [fnm Documentation](https://github.com/Schniz/fnm)

---

## 🐛 Known Issues

### Critical (Blocks Compilation)
- Duplicate `VenConfig` and `RuntimeConfig` struct definitions
  - **Location:** `src/core/mod.rs` vs `src/core/config.rs`
  - **Impact:** Won't compile
  - **Fix:** See `BUG_FIX_NEEDED.md`

### Minor (Post-Fix)
- Windows path handling may need testing
- Shell hook requires manual shell restart
- No async runtime yet (blocking HTTP calls)

---

## 📈 Metrics

**Code Statistics:**
- Total Lines: ~1,200
- Test Coverage: 6 unit tests + 20+ manual scenarios
- Supported Shells: bash, zsh, fish
- Supported Languages: Node.js (1), Python (planned), Go (planned)

**Dependencies:**
- clap 4 — CLI framework
- tokio 1 — Async runtime
- reqwest 0.12 — HTTP client
- serde 1 + toml 0.8 — Serialization
- rusqlite 0.31 — Local cache (Phase 2)
- colored + indicatif — Terminal output

---

## 📄 License

MIT License — See LICENSE file

---

## 🙏 Acknowledgments

Built on the shoulders of giants:
- [fnm](https://github.com/Schniz/fnm) — Fast Node Manager
- [npm](https://www.npmjs.com/) — Package registry
- [Rust](https://www.rust-lang.org/) — Systems programming language
- [pyenv](https://github.com/pyenv/pyenv), [rustup](https://rustup.rs/), etc.

---

## 📞 Contact

**Issues:** [GitHub Issues](https://github.com/yourusername/ven/issues)  
**Discussions:** [GitHub Discussions](https://github.com/yourusername/ven/discussions)

---

**Last Updated:** March 25, 2026  
**Current Milestone:** Phase 1 Complete (Testing Pending)  
**Next Milestone:** Phase 2 — Intelligence Layer
