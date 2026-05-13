# Ven — Changelog Page (`/changelog`)

## Page Goal

Show version history clearly. Developer scans it in 30 seconds and knows exactly what changed and when.

---

## Page Layout

```
Header
│
├── Section 1: Header + Filter Bar
├── Section 2: Version Timeline
└── Section 3: Footer Links
```

---

## Section 1: Header + Filter Bar

```
Changelog

Every change to ven, documented.
Latest release: v1.0.0 — January 15, 2024

Filter: [All] [Major] [Minor] [Patch] [Security]

[Subscribe to releases →]    [GitHub Releases →]
```

---

## Section 2: Version Timeline

**Layout:** Vertical timeline, newest first

**Each entry:**
```
┌──────────────────────────────────────────────────┐
│  v1.x.x  ·  DATE  ·  [tag]          [↓ Download]│
├──────────────────────────────────────────────────┤
│  Changes grouped by type                         │
└──────────────────────────────────────────────────┘
```

**Tags:**
- 🟢 `MAJOR` — breaking changes
- 🔵 `MINOR` — new features
- ⚪ `PATCH` — bug fixes
- 🔴 `SECURITY` — security fixes

---

## All Versions

---

### **v1.0.0** · January 15, 2024 · 🟢 MAJOR

```
Initial stable release.

✨ New
  • Node.js runtime management
  • Python runtime management
  • Go runtime management
  • Rust runtime management
  • Java (JDK) runtime management
  • Ruby runtime management
  • Deno runtime management
  • Bun runtime management
  • ven.toml project configuration
  • Automatic shell activation on cd
  • ven setup — shell hook installation
  • ven init — interactive project setup
  • ven status — environment observability
  • ven add / remove / upgrade — package management
  • ven-launcher — no-admin terminal spawner
  • PowerShell support (5.1 and 7+)
  • Bash / Zsh / Fish support
  • SHA256 verification on all downloads
  • Version alias resolution (20 → 20.20.2)
  • Per-terminal environment isolation
  • VEN_*_VERSION environment markers
  • Cross-platform: Windows, macOS, Linux

[↓ Download v1.0.0]    [Full diff →]
```

---

### **v0.9.0** · December 28, 2023 · 🔵 MINOR

```
Bun and Ruby support. Package management improvements.

✨ New
  • Bun runtime support (install, activate, packages)
  • Ruby runtime support (gem-based package management)
  • ven upgrade --apply flag
  • ven status --json output mode
  • GEM_HOME / GEM_PATH environment injection
  • BUN_INSTALL environment injection

🐛 Fixed
  • npm subprocess not found on Windows (#47)
  • ven.toml not updated after failed install (#51)
  • Path resolution on Windows with spaces (#53)
  • Python venv not activating on Zsh (#55)

[↓ Download v0.9.0]    [Full diff →]
```

---

### **v0.8.0** · December 10, 2023 · 🔵 MINOR

```
Deno and Java support. Shell integration improvements.

✨ New
  • Deno runtime support (single binary model)
  • Java (JDK) runtime support (JAVA_HOME management)
  • ven shell activate / deactivate commands
  • ven deactivate — safe overlay removal
  • Fish shell support
  • DENO_DIR environment injection
  • JAVA_HOME auto-set on activation
  • ven status --verbose mode

🐛 Fixed
  • Shell hook duplicate entries on re-running setup (#38)
  • Version resolution failing for patch versions (#41)
  • Go GOPATH not set correctly on Windows (#43)

⚡ Improved
  • Activation speed improved from ~200ms to <50ms
  • Download progress bar now shows speed + ETA

[↓ Download v0.8.0]    [Full diff →]
```

---

### **v0.7.0** · November 22, 2023 · 🔵 MINOR

```
Rust and Go support. ven-launcher introduced.

✨ New
  • Rust runtime support (cargo-based packages)
  • Go runtime support (go mod aware)
  • ven-launcher — standalone terminal spawner
  • CARGO_HOME / GOROOT / GOPATH injection
  • ven use <lang> <version> --global
  • Multiple language support in single ven.toml
  • ven list --verbose mode
  • ven list --json mode

🐛 Fixed
  • PATH not restored on deactivate (#29)
  • ven.toml not found in deeply nested directories (#31)
  • Binary verification failing on macOS arm64 (#33)

[↓ Download v0.7.0]    [Full diff →]
```

---

### **v0.6.0** · November 5, 2023 · 🔵 MINOR

```
Python support. venv management.

✨ New
  • Python runtime support
  • Automatic pip installation post-extract
  • venv creation and management
  • [venv] section in ven.toml
  • PYTHONHOME environment injection
  • ven add / remove for Python (pip-based)
  • requirements.txt sync

🐛 Fixed
  • Node npm path not found on fresh installs (#19)
  • ven.toml packages section not updating (#22)
  • SHA256 verification failing on partial downloads (#24)

⚡ Improved
  • Error messages now include fix suggestions
  • Download retry on network failure (3 attempts)

[↓ Download v0.6.0]    [Full diff →]
```

---

### **v0.5.0** · October 18, 2023 · 🔵 MINOR

```
Package management. ven add / remove / upgrade.

✨ New
  • ven add <package>
  • ven add <package>@<version>
  • ven remove <package>
  • ven upgrade
  • ven upgrade <package> --apply
  • Dependency conflict warning on remove
  • ven.toml [packages] sync on all operations
  • npm subprocess with ven-managed Node path

🐛 Fixed
  • Auto-activation not firing on PowerShell 5.1 (#12)
  • Version aliases not resolving on offline machines (#14)

[↓ Download v0.5.0]    [Full diff →]
```

---

### **v0.4.0** · October 2, 2023 · 🔵 MINOR

```
Shell integration. Auto-switching on cd.

✨ New
  • ven setup — installs shell hooks
  • Auto-activation on cd (PowerShell + Bash + Zsh)
  • ven shell activate (manual mode)
  • VEN_NODE_VERSION environment marker
  • VEN_ACTIVE environment marker
  • Nearest ven.toml wins (directory tree walk)
  • Global default via ~/.ven/versions/

🐛 Fixed
  • ven.toml not detected in parent directories (#8)

[↓ Download v0.4.0]    [Full diff →]
```

---

### **v0.3.0** · September 15, 2023 · 🔵 MINOR

```
ven init and ven status.

✨ New
  • ven init — interactive project setup
  • ven init --template
  • ven status (basic + verbose)
  • ven.toml [env] section support
  • Environment variable injection on activation
  • Version validation hints in init

[↓ Download v0.3.0]    [Full diff →]
```

---

### **v0.2.0** · September 1, 2023 · 🔵 MINOR

```
ven.toml support. Version resolution engine.

✨ New
  • ven.toml project configuration
  • [runtime] section parsing
  • Version alias resolution (20 → 20.20.2)
  • ven list command
  • ven list --verbose
  • Multiple Node.js versions coexist
  • ~/.ven/ storage structure

🐛 Fixed
  • Download failing on slow connections (#3)

[↓ Download v0.2.0]    [Full diff →]
```

---

### **v0.1.0** · August 15, 2023 · 🟢 MAJOR

```
Initial release. Node.js only.

✨ New
  • ven install node <version>
  • Downloads from nodejs.org official source
  • SHA256 checksum verification
  • Extracts to ~/.ven/node/<version>/
  • Binary validation after install
  • Windows (x64) + Linux (x64) + macOS (x64 + arm64)

This is the foundation. Everything builds from here.

[↓ Download v0.1.0]    [Full diff →]
```

---

## Section 3: Footer Links

```
─────────────────────────────────────────

All releases on GitHub →
RSS feed →
Subscribe to release notifications →

Versioning: ven follows semantic versioning (semver.org)
  MAJOR → breaking changes
  MINOR → new features, backward compatible
  PATCH → bug fixes only
```

---

## Design Notes

### **Visual style:**
- Dark background
- Timeline: vertical line on left, version dots
- Each version: card with subtle border
- Tag badges: colored pills
- Latest version: highlighted / pinned at top

### **Behavior:**
- Filter buttons hide/show versions by tag
- Each version card collapses to one line (click to expand)
- Download button links to GitHub release asset
- Diff link goes to GitHub compare view

### **Mobile:**
- Timeline line hidden
- Cards full width
- Filter bar scrolls horizontally

---

## Summary

| Section | Content |
|---------|---------|
| Header | Title + latest version + filter + links |
| Timeline | All versions newest → oldest |
| Each version | New / Fixed / Improved grouped cleanly |
| Footer | GitHub releases + RSS + semver note |

**Total versions shown: 10 (v0.1.0 → v1.0.0)**
**Core design: Vertical timeline, filtered by tag, expandable cards**