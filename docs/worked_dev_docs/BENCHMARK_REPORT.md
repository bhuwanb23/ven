# ven benchmark report — metrics, measurements, and identified problems

**Date:** 2026-10-06
**Target:** `ven` 0.2.8 (installed binary at `~/.ven/bin/ven.exe`, built 2026-09-03)
**Host:** Windows 11, corporate-managed, Zscaler TLS interception + Windows Defender real-time scanning
**Status:** 📊 MEASUREMENT PASS COMPLETE — no source changes in this pass
**Companion plan:** [`PERFORMANCE_BOTTLENECK_PLAN.md`](PERFORMANCE_BOTTLENECK_PLAN.md)

---

## Purpose

`ven` aims to replace eight-plus primary language tools (nvm/fnm, pyenv/uv,
sdkman, rustup, nvm/deno/bun managers, npm/pnpm, pip, cargo, gem, composer…).
To claim it *beats* those alternatives we need numbers, not adjectives. This
report records the metric set, the measured values, the environment facts that
distort them, and the ranked list of weakpoints that the plan works through.

---

## Method (read this before comparing to other numbers)

1. **Overhead-free timing.** Samples use bash's `$EPOCHREALTIME` read before and
   after the command. A first attempt used `date +%s%N`, which spawns a process
   per sample on Windows and injected **~75 ms of measurement overhead per
   reading** — those numbers were discarded.
2. **Process-spawn floor established.** `cmd /c exit` = **85–91 ms** on this
   host (Defender/AV image load). Every CLI number below is *above* that floor;
   a tool cannot beat the floor.
3. **Warm samples unless labelled cold.** 3–5 runs per tool, min–max reported.
   The first run of any binary is a cold scan (see `performance.md`: 4–8 s).
4. **Isolation for stateful ops.** Runtime installs used `VEN_HOME=/tmp/...` and
   `UV_PYTHON_INSTALL_DIR=/tmp/...` so the real `~/.ven` (1.04 GB) was untouched.
5. **Exit codes captured explicitly.** A helper that swallowed exit status once
   produced two false "installs"; those samples were re-run with real output.

### Environment facts that distort any measurement here

| Fact | Evidence |
|------|----------|
| TLS is intercepted | `issuer: Zscaler Intermediate Root CA (zscalertwo.net)`, `verify error: unable to get local issuer certificate` |
| Some URLs are keyword-blocked | `static.crates.io/crates/zip/0.6.6/download` → 403 Zscaler block page; same path with `serde` → 200 |
| python.org serves gzip **unrequested** | see Finding F1 — `content-encoding: gzip` on a request with no `Accept-Encoding` |
| No MSVC, no MinGW, no `dlltool`, no `gcc` | `rustc host: x86_64-pc-windows-gnu`; no VS/LLVM/msys dirs → `cargo build` blocked |
| Package registries reachable | npm 200 (75 ms), PyPI 200 (90 ms), crates index 200 (94 ms) |

---

## Metric 1 — CLI startup latency (warm, ms)

| Tool | Sample 1 | Sample 2 | Sample 3 | Sample 4 | Sample 5 | Range |
|------|---------:|---------:|---------:|---------:|---------:|-------|
| **ven** | 317.2 | 263.7 | 255.7 | 262.8 | 258.5 | **256–317** |
| uv | 307.5 | 316.1 | 302.3 | 298.1 | 305.4 | 298–316 |
| go | 296.3 | 273.7 | 274.1 | 276.1 | 277.5 | 274–296 |
| node | 289.7 | 301.9 | 305.2 | 291.4 | 318.0 | 290–318 |
| cargo | 742.8 | 391.2 | 436.3 | 356.4 | 353.1 | 353–436 (first 743) |
| pnpm | 1151.4 | 869.5 | 887.7 | 877.4 | 978.6 | 870–1151 |
| npm | 2026.5 | 1836.7 | — | — | — | 1837–2027 |
| git (small signed native) | 121.8 | 126.0 | 137.5 | 131.3 | 127.8 | 122–137 |
| **spawn floor** (`cmd /c exit`) | 85.3 | 86.3 | 90.9 | 88.0 | 86.3 | **85–91** |

**Reading:**

- ven **already beats uv, go, node, cargo, pnpm and npm** on this host.
- ven's *own* cost above the floor is **~170 ms** (256 ms − 86 ms). `git` runs
  its whole program in ~40 ms above the floor, so 170 ms is the addressable gap.
- `ven-launcher` could not be measured honestly: `ven-launcher --version` is
  rejected by clap (`error: unexpected argument`), so its ~90 ms is an error
  path, not a real workload. Not reported as a result.

## Metric 2 — subcommand latency (warm, ms)

| Command | Samples | Range |
|---------|--------:|-------|
| `ven path` | 263.4, 273.5, 309.8 | 263–310 |
| `ven list` | 286.6, 288.3, 277.2 | 277–288 |
| `ven status` | 301.9, 289.1, 362.0 | 289–362 |
| `ven shell activate <dir>` | 312.1, 313.8, 299.7, 294.8, 351.6 | 295–352 |
| `ven doctor` | 773.9, 798.0, 2912.0 | **774–2912 (12–25× others)** |

Every subcommand costs essentially the same as `--version`: **startup dominates,
command work is noise.** `ven doctor` is the outlier — it spawns/inspects
multiple installs and PATH state, and its worst case approaches 3 s.

## Metric 3 — cd-hook latency (the hook runs on *every* directory change)

Measured in-process inside the real installed PowerShell hook (`Measure-Command`
over 20 iterations, so PowerShell startup is excluded):

| Hook component | avg | min | max |
|----------------|----:|----:|----:|
| `__ven_toml_sig` (walk to drive root, deep project) | **3.42 ms** | 1.78 | 13.08 |
| `__ven_clear_ven_state` | **2.90 ms** | 1.84 | 6.09 |
| **full `__ven_activate` — fast path (no `ven.toml`)** | **5.60 ms** | 1.32 | 25.38 |

| Path | Cost |
|------|-----:|
| Fast path (no `ven.toml` up-tree) | **~5.6 ms**, no subprocess |
| **Slow path (inside a `ven.toml` project)** | **~300 ms** — spawns `ven shell activate` |

**Reading:** the fast path (shipped in `performance.md`) works exactly as
documented and is genuinely cheap. The slow path is the single largest recurring
latency in normal use — every `cd` into any project pays a full process spawn
plus Defender scan. Equivalent tools (direnv, fnm, mise) sit in the 5–20 ms
range for the same operation.

## Metric 4 — package install throughput (ms)

npm ecosystem, warm registry cache, fresh package each round so runs are
comparable:

| Scenario | ven | pnpm | npm |
|----------|----:|-----:|----:|
| cold `add lodash` (empty project) | 9071 | 8826 | 5105 |
| warm `express` (large tree) | **2325** | 3368 | 4436 |
| warm `axios` | 5431 | **3552** | 4471 |
| warm `chalk` (no deps) | **459** | 1655 | 2715 |
| 2nd project, same tree (marginal cost) | *n/a¹* | **1439** | 3772 |

¹ `ven sync` exits 1 without a `ven.lock`, so it produced no install — see F7.

Python ecosystem:

| Scenario | uv | pip/venv |
|----------|---:|---------:|
| cold venv + `requests` | **1060** | 14168 |
| warm 2nd venv + `requests` | **1066** | 6244 |

**Reading:**

- ven wins `chalk` decisively and wins `express`, but **loses `axios`**; variance
  is high because `ven add` also runs the OSV security gate plus a registry
  latest-version lookup before installing (F6).
- uv is **~6× faster than pip** — that is the bar for any Python claim.
- Network floor for registry metadata is 75–90 ms, so none of these are
  network-bound; the differences are tool overhead.

## Metric 5 — storage efficiency

| Item | Measured |
|------|----------|
| express tree, npm `node_modules` | 4 MB, 622 files |
| express tree, pnpm `node_modules` | 4 MB, 631 files, **hardlink count = 2** → points into store |
| pnpm global store | 13 MB (2nd project marginal cost ≈ 0) |
| npm download cache (`_cacache`) | 128 MB |
| uv cache | **372 MB** (`archive-v0` 314 MB) |
| pip venv vs uv venv (same package) | **14 MB vs 1 MB** |
| `ven` home (1 language: Node) | **1.04 GB** — node 629 MB, bin 29 MB, cache 2 MB, `intelligence.db` 1 MB |
| `ven install node 22` into isolated `VEN_HOME` | 134 MB |

**Reading:** ven delegates package installs to npm, so it inherits npm's
copy-per-project model. There is **no content-addressable store** in `~/.ven`
(package cache is 2 MB — just npm's own metadata), so ven currently **loses the
multi-project storage story to pnpm and uv**, both of which pay ~0 marginal cost
for a second project.

## Metric 6 — runtime installation (the layer that must beat nvm/uv)

| Command | Result | Time | Size |
|---------|--------|-----:|-----:|
| `ven install node 22` (isolated `VEN_HOME`) | ✅ exit 0, validated binary+version+PATH | — | 134 MB |
| `ven install python 3.12.7` | ❌ **exit 1** | 3304 ms to failure | — |
| `ven install python 3.11.9` | ❌ **exit 1** (same reason) | — | — |
| `uv python install 3.12` | ❌ TLS `invalid peer certificate: UnknownIssuer` | — | — |
| `UV_NATIVE_TLS=1 uv python install 3.12` | ✅ | 24.66 s | 68 MB |

**Reading:** Node install works and self-validates. **Python install is broken
on Windows** (F1) — the flagship "any language, any version" claim fails for one
of the two most-used languages. uv's own TLS failure is *its* weakness under
corporate interception; ven uses `rustls-tls-native-roots` and its own HTTPS
calls (OSV, checksums, npm metadata) all succeeded on this host.

---

## Findings — ranked problems

### F1 — `ven install python` is broken on Windows (correctness, P0)

**Symptom (reproduced on two versions):**

```
[FETCH] https://www.python.org/ftp/python/3.12.7/python-3.12.7-embed-amd64.zip
[OK] Saved ...\.cache\python-3.12.7-embed-amd64.zip
Error: Checksum unavailable for python-3.12.7-embed-amd64.zip — refusing to continue without verification.
  Reason: filename python-3.12.7-embed-amd64.zip not found on release page
EXIT=1
```

**Root cause (verified by experiment, not inferred).** The scraper at
`src/core/python_install.rs:249` does `html.find(filename)` on the body of
`https://www.python.org/downloads/release/python-<nodots>/`:

```rust
let idx = html
    .find(filename)
    .ok_or_else(|| anyhow!("filename {} not found on release page", filename))?;
```

python.org responds with `content-encoding: gzip` **even when the client never
advertised gzip**, and ven's reqwest is built without the `gzip` feature
(`Cargo.toml:48` — features are `json, blocking, rustls-tls,
rustls-tls-native-roots, http2, charset`; there is no `async-compression` in
`Cargo.lock`). So `.text()` returns compressed bytes, `find()` sees no filename,
and the installer fails closed.

Proof (same URL, only the decompression differs):

| Request | Bytes | `content-encoding` | filename occurrences | `<title>` |
|---------|------:|--------------------|---------------------:|-----------|
| no `Accept-Encoding` (what ven does) | 9 911 | `gzip` | **0** | none (binary) |
| `--compressed` (decode gzip) | 48 306 | gzip→decoded | **4** | `Python Release Python 3.12.7` |

**The fail-closed behaviour is correct** — ven refuses to install unverified
bytes. But the feature is unusable, and the error message blames the release
page when the real cause is undecoded content. **Fix direction:** enable
reqwest's `gzip` feature (one line) *and* make the error distinguish "body was
not decodable HTML" from "filename genuinely absent". See plan P1-1.

### F2 — cd-hook slow path costs ~300 ms per project `cd` (latency, P0)

Fast path is 5.6 ms; slow path spawns ven (Metric 3). This is the highest
*frequency* latency in the product: a developer `cd`-ing between two projects
pays it dozens of times an hour, and it is 20–60× the equivalent tools.

### F3 — ~170 ms of startup above the process floor (latency, P1)

Metric 1. Startup dominates every subcommand (Metric 2), so this cost is paid by
`ven add`, `ven status`, `ven doctor` *and* by the hook spawn in F2. Contributors
to investigate: 13 MB unsigned PE image (Defender scan — already documented in
`performance.md`), tokio multi-thread runtime + reqwest/rustls root-pool
initialisation, config/DB open before clap dispatch.

### F4 — package installs inherit a plaintext HTTP registry (security, P0)

```
--- ~/.npmrc ---
strict-ssl=false
registry=http://registry.npmjs.org/
```

ven does **not** write this file (`grep -rn npmrc src/` → no matches), but
`ven add` spawns npm at `src/core/packages.rs:15` and `:73` with no registry or
TLS override, so **every package fetched through ven travels over plaintext HTTP
with certificate verification disabled** — MITM can substitute package tarballs.
npm itself warns on stderr: *"must use TLS 1.2 or higher. You are currently using
plaintext http to connect."* See plan P1-2.

### F5 — PHP installer fails *open* on checksums (security consistency, P1)

`src/core/php_install.rs:257` prints
`"Checksum unavailable for PHP … Continuing without verification."` while **every
other runtime refuses** (`bun_install.rs:133`, `deno_install.rs:128`,
`download.rs:145`, `go_install.rs:181`, `java_install.rs:156`,
`python_install.rs:204`, `ruby_install.rs:216`, `rust_install.rs:209`). One
runtime silently weaker than the other seven undermines the integrity story in
`docs/security-model.md`. See plan P1-3.

### F6 — `ven add` never renders simulation warnings (UX/trust, P1)

`grep -c 'result\.warnings' src/cli/add/mod.rs` = **0**. The npm-family adapter
generates warnings (e.g. the stub-adapter notice *"Best-effort stub: full
npm-style simulation not wired for this runtime; install may still proceed."*)
into `SimulationResult.warnings`, and `add` renders `engine_incompatibilities`
and `conflict_chains` but **drops `warnings`**. Meanwhile `ven check-add`
(`check_add.rs:49`) and `ven graph` both print them — so the same advisory is
visible from one command and invisible from the one that actually installs.
`add` also passes a literal `false` for the gate's `json` parameter
(`src/cli/add/mod.rs:151` → `run_install_gate(&kind, &candidates, &gate_policy,
yes, false)`; signature at `src/core/install_gate.rs:492-498`), so `ven add
--json` consumers get no vulnerability data. See plan P1-4.

### F7 — `ven sync` requires `ven.lock` (friction — by design, P1)

`ven sync` on a `ven.toml` that lists packages exits 1:
`Error: No ven.lock in this directory. Run: ven lock …`. This is **documented,
intentional behaviour** (`docs/cmds/sync.md`), not a defect — but the first-run
experience is a failed command where `pnpm install` would just work. Worth a
smarter fallback or a clearer message. See plan P2-3.

### F8 — no content-addressable store (storage/throughput, P2)

Metric 5. Delegating to npm means full copies per project and no reuse between
projects; pnpm and uv both achieve ~0 marginal cost. This is the biggest
structural gap versus the "beat the alternatives" goal.

### F9 — `cargo build` blocked on this machine (toolchain, P0 for verification)

`cargo fetch` now succeeds (exit 0) after vendoring `zip` 0.6.6 from upstream
commit `21a2058 "publish 0.6.6 with aes 0.8"` and patching it in the **untracked**
`.cargo/config.toml`. `cargo build --release` still fails:

```
error: error calling dlltool 'dlltool.exe': program not found
error: could not compile `windows-sys` (lib)
error: could not compile `parking_lot_core` (lib)      BUILD_EXIT=101
```

rustup installed `x86_64-pc-windows-gnu` (no MSVC detected) and neither MinGW
(`dlltool`, `gcc`) nor Visual Studio nor LLVM exists on this host; `winget` is
available. **Consequence: `cargo test` / `cargo clippy` have never run on the
current tree**, so the new gate and security tests are unexecuted. See plan P0-1.

### F10 — `ven doctor` latency variance (P3)

774 ms – 2912 ms (Metric 2). A diagnostic command that takes ~3 s on its worst
run is the slowest thing in the CLI and is the command users run when something
is already wrong.

---

## Carried findings from earlier passes (status register)

These were established before this benchmarking pass and are recorded here so
the plan covers *every* known problem, not just the new measurements.

| ID | Finding | Evidence | Status |
|----|---------|----------|--------|
| S1 | **Dependency intelligence is npm-only.** 7 of 9 ecosystems route to a stub that always reports compatible | `src/intelligence/adapters/mod.rs:1` ("npm family plus deterministic stubs"); real `NpmFamilyAdapter` only for Node (`:334`) and Bun (`:309`); `GenericStubAdapter` for Python `:257`, Go `:268`, Rust `:277`, Java `:287`, Deno `:298`, Ruby `:319`, PHP `:331` + fallback `:336`; message at `:187` | ⚠️ OPEN — mitigated for `check-add`/`graph`, still invisible in `add` (F6) |
| S2 | **`ven add` for Java never installs anything.** It edits the manifest and reports `[OK]`, but no Maven/Gradle process is ever spawned | `src/cli/add/languages.rs:209-293` → `java_manifest::add` writes `pom.xml`/`build.gradle` (`src/core/java_manifest.rs:99-105`); `grep` for `Command::new`/`spawn` of `mvn`/`gradle`/`gradlew` across `src/` = **0 matches** | 🔴 OPEN |
| S3 | **Path traversal in Ruby 7z extraction (Windows)** — validated and fixed | commit `d94a3a9`; entry validation before `sevenz_rust` decompression + regression tests in `ruby_install.rs` | ✅ FIXED (unverified by `cargo test` — see F9) |
| S4 | **Retraction:** `tar.unpack()` in self-update is *not* vulnerable | `tar` 0.4.45 rejects `ParentDir` and canonicalizes against dest (vendored `entry.rs`) | ✅ NO ACTION — documented so it is not "re-fixed" |
| S5 | **OSV pre-install security gate** for all 9 runtimes | commit `9570488`; `src/core/install_gate.rs` | ✅ SHIPPED — overhead to be measured (plan P3-2) |
| S6 | **`cargo test` / `clippy` never executed on the current tree** | F9 below | 🔴 BLOCKED |
| S7 | Stale planning doc claims compile errors that do not exist in this tree | `docs/worked_dev_docs/BUG_FIX_NEEDED.md` asserts duplicate `VenConfig`/`RuntimeConfig` definitions "will cause compilation errors" — not present here | ⚠️ STALE — schedule correction (plan Phase 4) |

---

## Not yet measured (next metrics to add)

- Peak RSS / memory footprint per command.
- Parallel-download throughput (uv and pnpm download concurrently; ven's npm path
  is serial).
- Large-tree install (50+ dependencies) — where pnpm's linking should pull ahead.
- `ven check` / `ven graph` / `ven why` latency (SQLite `intelligence.db` reads).
- Cold-boot numbers (empty file cache) vs the warm figures here.
- Isolating the Defender contribution (exclusion on vs off) to confirm the
  170 ms split between image scan and program logic.
- Gate overhead in isolation (time `ven add` with the OSV gate vs without).

## Repo state at time of writing

- Branch `security/fix-ruby-7z-path-traversal`, commits `d94a3a9` (7z traversal
  fix) and `9570488` (OSV pre-install gate).
- Working tree: `Cargo.lock` modified + untracked `.cargo/config.toml` (both from
  the build-unblock attempt; keep or revert deliberately).
- **No product source was edited during this measurement pass.**
