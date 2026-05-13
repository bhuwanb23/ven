# Ven — Languages Page (`/languages`)

## Page Goal

Show all supported languages, what ven manages for each, and what's coming next. Fast, visual, no fluff.

---

## Page Layout

```
Header
│
├── Section 1: Hero Bar
├── Section 2: Language Grid (8 cards)
├── Section 3: Per-Language Detail (click to expand)
├── Section 4: Coming Soon
└── Section 5: Request a Language
```

---

## Section 1: Hero Bar

```
8 Languages. One Tool. Zero Conflicts.

ven manages runtimes, packages, and environments
for every language below — with the same commands.

[Node.js] [Python] [Go] [Rust] [Java] [Ruby] [Deno] [Bun]
```

---

## Section 2: Language Grid

**8 cards in a responsive grid (4x2 desktop, 2x4 tablet, 1x8 mobile)**

Each card shows:

```
┌─────────────────────────┐
│  [Logo]                 │
│                         │
│  Node.js                │
│  v18 • v20 • v22        │
│                         │
│  Package Manager: npm   │
│  Config: package.json   │
│  Status: ● Stable       │
│                         │
│  [View Details →]       │
└─────────────────────────┘
```

### **All 8 Cards:**

**Node.js**
```
Logo:    Node.js official logo
Versions: v18 • v20 • v22
Package:  npm
Config:   package.json
Status:   ● Stable
```

**Python**
```
Logo:    Python official logo
Versions: 3.10 • 3.11 • 3.12
Package:  pip + venv
Config:   requirements.txt
Status:   ● Stable
```

**Go**
```
Logo:    Go gopher logo
Versions: 1.20 • 1.21 • 1.22
Package:  go mod
Config:   go.mod
Status:   ● Stable
```

**Rust**
```
Logo:    Rust logo
Versions: 1.73 • 1.74 • 1.75
Package:  cargo
Config:   Cargo.toml
Status:   ● Stable
```

**Java**
```
Logo:    Java logo
Versions: JDK 11 • 17 • 21
Package:  Maven / Gradle
Config:   pom.xml
Status:   ● Stable
```

**Ruby**
```
Logo:    Ruby logo
Versions: 3.1 • 3.2 • 3.3
Package:  gem + bundler
Config:   Gemfile
Status:   ● Stable
```

**Deno**
```
Logo:    Deno logo
Versions: 1.38 • 1.39 • 1.40
Package:  native / npm:
Config:   deno.json
Status:   ● Stable
```

**Bun**
```
Logo:    Bun logo
Versions: 1.0 • 1.1
Package:  bun (npm-compatible)
Config:   package.json
Status:   ● Stable
```

---

## Section 3: Per-Language Detail Panel

**Click any card → expands detail panel below grid**

### **Node.js Detail:**

```
Node.js

The most widely used JavaScript runtime.
ven manages Node versions per-project and
uses npm for package operations.

Install:
  ven install node 20
  ven install node 22

ven.toml:
  [runtime]
  node = "20"

  [packages]
  express = "4.18.2"
  lodash  = "*"

What ven sets:
  PATH  → ~/.ven/node/20.20.2/bin

Package operations:
  ven add express       → npm install express
  ven remove lodash     → npm uninstall lodash
  ven upgrade react     → npm update react

Includes:   node  npm  npx
Downloads:  nodejs.org/dist/

[Read Node.js docs →]
```

---

### **Python Detail:**

```
Python

General-purpose language with pip for packages
and venv for project isolation. ven handles
venv creation and activation automatically.

Install:
  ven install python 3.11
  ven install python 3.12

ven.toml:
  [runtime]
  python = "3.11"

  [packages]
  flask   = "3.0.0"
  requests = "*"

  [venv]
  path = ".venv"

What ven sets:
  PATH        → ~/.ven/python/3.11.5/bin
  PYTHONHOME  → ~/.ven/python/3.11.5

Package operations:
  ven add flask         → pip install flask
  ven remove requests   → pip uninstall requests
  ven upgrade django    → pip install --upgrade django

Includes:   python  pip  venv
Downloads:  python.org/downloads/

[Read Python docs →]
```

---

### **Go Detail:**

```
Go

Compiled language from Google. ven manages
Go versions and sets GOROOT/GOPATH. Package
management handled natively by go mod.

Install:
  ven install go 1.21

ven.toml:
  [runtime]
  go = "1.21"

What ven sets:
  PATH    → ~/.ven/go/1.21.5/bin
  GOROOT  → ~/.ven/go/1.21.5
  GOPATH  → ~/go

Package operations:
  Handled by go mod (native)
  go get github.com/gin-gonic/gin

Includes:   go  gofmt  godoc
Downloads:  go.dev/dl/

[Read Go docs →]
```

---

### **Rust Detail:**

```
Rust

Systems language with Cargo as its all-in-one
build system and package manager. ven manages
Rust toolchain versions.

Install:
  ven install rust 1.75

ven.toml:
  [runtime]
  rust = "1.75"

  [packages]
  serde    = "1.0"
  tokio    = "1.35"

What ven sets:
  PATH        → ~/.ven/rust/1.75.0/bin
  CARGO_HOME  → ~/.ven/rust/1.75.0

Package operations:
  ven add serde         → cargo add serde
  ven remove tokio      → cargo remove tokio
  ven upgrade serde     → cargo update serde

Includes:   rustc  cargo  rustfmt  clippy
Downloads:  static.rust-lang.org

[Read Rust docs →]
```

---

### **Java Detail:**

```
Java

Enterprise-grade language. ven manages JDK
versions and sets JAVA_HOME automatically.
Package management via Maven or Gradle (user-managed).

Install:
  ven install java 17
  ven install java 21

ven.toml:
  [runtime]
  java = "17"

What ven sets:
  PATH       → ~/.ven/java/17.0.9/bin
  JAVA_HOME  → ~/.ven/java/17.0.9

Package operations:
  Handled by Maven or Gradle (native)
  mvn install  /  gradle build

Includes:   java  javac  jar  jshell
Downloads:  adoptium.net

[Read Java docs →]
```

---

### **Ruby Detail:**

```
Ruby

Dynamic language popular for web (Rails) and
DevOps tooling. ven manages versions and
uses gem/bundler for packages.

Install:
  ven install ruby 3.2

ven.toml:
  [runtime]
  ruby = "3.2"

  [packages]
  rails  = "7.1.0"
  sinatra = "*"

What ven sets:
  PATH      → ~/.ven/ruby/3.2.2/bin
  GEM_HOME  → ~/.ven/ruby/3.2.2
  GEM_PATH  → ~/.ven/ruby/3.2.2

Package operations:
  ven add rails         → gem install rails
  ven remove sinatra    → gem uninstall sinatra
  ven upgrade rails     → gem update rails

Includes:   ruby  gem  bundler  irb
Downloads:  rubyinstaller.org / ruby-lang.org

[Read Ruby docs →]
```

---

### **Deno Detail:**

```
Deno

Modern JavaScript/TypeScript runtime by the
creator of Node.js. Single binary, no package
manager needed. Imports via URL natively.

Install:
  ven install deno 1.40

ven.toml:
  [runtime]
  deno = "1.40"

What ven sets:
  PATH      → ~/.ven/deno/1.40.0
  DENO_DIR  → ~/.cache/deno

Package operations:
  Handled natively by Deno
  Import URLs directly in code
  deno.json manages import maps

  npm-compatible:
  import express from "npm:express@4.18.2"

Includes:   deno (single binary)
Downloads:  github.com/denoland/deno/releases

[Read Deno docs →]
```

---

### **Bun Detail:**

```
Bun

Fast all-in-one JavaScript runtime.
Drop-in Node.js replacement. npm-compatible.
Single binary like Deno, package.json like Node.

Install:
  ven install bun 1.0

ven.toml:
  [runtime]
  bun = "1.0"

  [packages]
  express = "4.18.2"
  lodash  = "*"

What ven sets:
  PATH        → ~/.ven/bun/1.0.20
  BUN_INSTALL → ~/.ven/bun/1.0.20

Package operations:
  ven add express       → bun add express
  ven remove lodash     → bun remove lodash
  ven upgrade react     → bun update react

Includes:   bun (single binary — runtime + bundler + test runner)
Downloads:  github.com/oven-sh/bun/releases

Switch from Node:
  Change node = "20" → bun = "1.0"
  Same package.json. Same code. 10x faster.

[Read Bun docs →]
```

---

## Section 4: Coming Soon

```
Coming Soon

┌──────────┬──────────┬──────────┬──────────┐
│   PHP    │  Elixir  │  .NET    │   Zig    │
│          │          │          │          │
│ Composer │   Mix    │  NuGet   │  Single  │
│          │          │          │  binary  │
│  🔜      │  🔜      │  🔜      │  🔜      │
└──────────┴──────────┴──────────┴──────────┘

┌──────────┬──────────┬──────────┬──────────┐
│   Lua    │  Swift   │  Kotlin  │  Scala   │
│          │          │          │          │
│LuaRocks  │  Swift   │  Gradle  │   sbt    │
│          │  PM      │          │          │
│  🔜      │  🔜      │  🔜      │  🔜      │
└──────────┴──────────┴──────────┴──────────┘
```

---

## Section 5: Request a Language

```
Don't see your language?

We add languages based on community demand.

[Request a Language →]     ← Opens GitHub issue

Or contribute it yourself:
[Plugin System Guide →]    ← Goes to contributing docs

Most-requested:
  PHP      ████████████░░  342 votes
  Elixir   ████████░░░░░░  218 votes
  .NET     ██████░░░░░░░░  187 votes
  Swift    ████░░░░░░░░░░  143 votes
  Zig      ████░░░░░░░░░░  138 votes
```

---

## Comparison: How Each Language Handles Packages

```
Quick Reference

Language   Package Mgr    Isolation       Config
────────────────────────────────────────────────────
Node.js    npm            node_modules/   package.json
Python     pip + venv     .venv/lib/      requirements.txt
Go         go mod         GOPATH cache    go.mod
Rust       cargo          target/         Cargo.toml
Java       Maven/Gradle   ~/.m2/          pom.xml
Ruby       gem/bundler    GEM_HOME        Gemfile
Deno       native/URL     ~/.cache/deno/  deno.json
Bun        bun (npm-compat) node_modules/ package.json
```

---

## Page Design Notes

### **Behavior:**
- Grid cards are clickable
- Click card → smooth expand of detail panel below
- Only one detail panel open at a time
- Language logo animates on hover
- Status badge pulses green for stable

### **Filter bar (optional):**
```
Filter: [All] [Stable] [Planned] [Web] [Systems] [Scripting]
```

### **Mobile:**
- 1 column grid
- Cards stack vertically
- Detail panel expands inline
- Full width

---

## Summary

| Section | Content |
|---------|---------|
| Hero | 8 language chips, one-liner |
| Grid | 8 cards with key info |
| Detail | Click to expand full per-language info |
| Coming Soon | 8 planned languages with progress |
| Request | GitHub issue link + vote counts |

**Total sections: 5**
**Core section: Language grid + detail panel**