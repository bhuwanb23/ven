# Performance & bottleneck remediation plan

**Goal:** make `ven` measurably faster and strictly safer than the eight-plus
tools it replaces, on a real corporate Windows host — with numbers, not claims.
**Status:** 📋 PLANNED (Phase 0 partially unblocked)
**Baseline:** [`BENCHMARK_REPORT.md`](BENCHMARK_REPORT.md) (2026-10-06)
**Owner:** —

---

## Success criteria (all measured, all reproducible)

| # | Metric | Baseline today | Target |
|---|--------|---------------:|-------:|
| M1 | cd-hook inside a project (slow path) | **~300 ms** | **≤ 15 ms** (≤ 3× fast path) |
| M2 | cd-hook outside a project (fast path) | 5.6 ms | ≤ 5 ms (no regression) |
| M3 | `ven --version` warm | 256 ms | **≤ 150 ms** (floor is 86 ms) |
| M4 | `ven doctor` worst case | 2912 ms | ≤ 800 ms |
| M5 | `ven install python <ver>` | ❌ exit 1 | ✅ exit 0, checksum verified |
| M6 | Package fetch transport | plaintext HTTP, `strict-ssl=false` | HTTPS enforced, verification on |
| M7 | Checksum policy across runtimes | 7 of 8 fail closed, PHP fails open | 8 of 8 fail closed |
| M8 | Warnings surfaced by `ven add` | never rendered | rendered in text **and** `--json` |
| M9 | 2nd project marginal disk cost | full copy (npm model) | ≈ 0 (store + links) |
| M10 | `cargo test` / `cargo clippy -D warnings` | **never run on current tree** | green on every phase exit |
| M11 | `ven add` on Java project | manifest edit only, reported as installed | fetched, **or** clearly labelled manifest-only |
| M12 | Analysis provenance in `ven add --json` | absent (7 of 9 ecosystems silently stubbed) | explicit `analysis: real \| stub` + warning |

Every target must be re-measured with the same method as the baseline report
(`$EPOCHREALTIME`, warm samples, spawn floor stated) so before/after is
comparable.

---

## Principles

1. **Measure before changing.** No optimization without a number that shows it
   mattered; revert changes that do not move their metric.
2. **Never weaken a safety control for speed.** Checksums stay fail-closed; the
   fix for F1 is correct decoding, not skipping verification.
3. **Fail closed, and say why.** Where a control must refuse (F1, F5), the error
   must name the real cause, not a misleading one.
4. **No behaviour regressions** in the existing test suite — 291 `#[test]`
   functions counted statically (281 in `src/`, 10 in `tests/`), **none of which
   have ever executed on this host** — which is why P0-1 (toolchain) comes
   before code changes.
5. **Windows is the hard target.** Defender, TLS interception and process-spawn
   cost are the environment; they must be measured, not wished away.

---

## Phase 0 — Make verification possible (blocker for everything else)

### P0-1 Unblock `cargo test` / `cargo clippy` 🔴 BLOCKED, needs a decision

**Problem (F9).** `cargo fetch` is fixed (exit 0) via the vendored `zip` 0.6.6
patched from the **untracked** `.cargo/config.toml`, but `cargo build` dies with
`error calling dlltool 'dlltool.exe': program not found`. The host has a GNU
toolchain with no MinGW, no MSVC and no LLVM.

**Options (pick one):**

| Option | Command | Cost | Notes |
|--------|---------|------|-------|
| A. MSVC Build Tools | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --passive"` | several GB | matches how `ven.exe` was originally shipped |
| B. LLVM (gives `llvm-dlltool` + `clang`) | `winget install LLVM.LLVM` | ~500 MB | then still needs `-C linker=` wiring for `windows-gnu` |
| C. MinGW-w64 | `winget install MSYS2.MSYS2` + `pacman -S mingw-w64-x86_64-toolchain` | ~1 GB | gives both `dlltool` and `gcc` |

**Verification:**

```bash
cargo fetch                       # already exit 0
cargo build --release --no-default-features --bin ven   # target: exit 0
cargo test --workspace            # target: all green (incl. 15 gate tests +
                                   # 6 new security tests: 4 ruby_install +
                                   # 2 extract, alongside pre-existing ones)
cargo clippy --workspace --all-targets -- -D warnings   # target: exit 0
```

**Do not** start P1+ code edits before this passes — the new tests have never
run, and `extract.rs` is shared by every extractor.

### P0-2 Check in the benchmark suite so numbers stay reproducible

Add `scripts/bench.sh` (and a `.ps1` sibling) implementing the exact method from
the report: `$EPOCHREALTIME` timing, stated spawn floor, warm/cold split, N=5
with min–max, and these workloads: startup, subcommand set, hook fast/slow path,
`add` warm rounds, storage snapshot, runtime install into an isolated
`VEN_HOME`. Emit a markdown table so results can be pasted into the report.

**Verification:** running the script twice on an unchanged tree produces
comparable tables; numbers land within ~10 % of the report's baseline.

---

## Phase 1 — P0 correctness & security (smallest diffs, highest stakes)

### P1-1 Fix `ven install python` (F1)

**Change:** enable gzip decoding for ven's HTTP client —
`Cargo.toml:48`, add `"gzip"` to the reqwest feature list — so the python.org
body decodes before the scraper sees it.

**Also:** make the failure honest. `python_install.rs:249` should distinguish
"body did not contain a filename (possibly undecoded/HTML-changed)" from
"filename absent from the release page", and the message at `:204` should say
which happened. Keep **fail-closed**: never install without a verified hash.

**Verify:**

```bash
cargo test --workspace
VEN_HOME=/tmp/vb ven install python 3.12.7   # target: exit 0, checksum OK printed
VEN_HOME=/tmp/vb ven install python 3.11.9   # target: exit 0
```

Prove the fix is the decode by asserting the fetched body length > 40 000 bytes
in a unit test against a recorded fixture.

### P1-2 Force HTTPS + strict SSL on delegated package fetches (F4)

**Change:** in `src/core/packages.rs:15` and `:73` (the npm spawn sites), pass
`--registry https://registry.npmjs.org/` and `strict-ssl=true` so a user's
`~/.npmrc` cannot silently downgrade ven's installs to plaintext.

**Also:** add a `ven doctor` check that reports `registry` not starting with
`https://` or `strict-ssl=false`, with the exact fix line to paste.

**Verify:** with a deliberately broken `~/.npmrc` (the current one), `ven add`
must fetch over TLS (npm stderr warning gone) and `ven doctor` must flag the
file; with a clean `~/.npmrc`, behaviour and timings are unchanged.

### P1-3 Make PHP fail closed like the other seven runtimes (F5)

**Change:** `src/core/php_install.rs:257` — replace "Continuing without
verification" with the same refuse path used by `ruby_install.rs:216` /
`go_install.rs:181`.

**Verify:** unit test asserting `verify_php_archive` returns `Err` when no
checksum is obtainable; confirm `docs/security-model.md` now describes reality.

### P1-4 Surface simulation warnings from `ven add` (F6)

**Change:** render `result.warnings` in `add`'s success path, mirroring
`check_add.rs:49-53`, and include them in `add --json` (the gate currently
hardcodes `json: false`).

**Verify:** a Node project whose adapter emits a warning prints it under
`ven add`, matches `ven check-add`, and appears under `add --json`; a
stub-ecosystem project shows the "best-effort stub" notice instead of silence.

### P1-5 Make `ven add` for Java honest (S2)

**Problem.** `cmd_add_java` (`src/cli/add/languages.rs:209`) writes the
coordinate into `pom.xml`/`build.gradle` and prints `[OK] Added …`, then updates
`ven.toml` — but no `mvn`/`gradle` process is ever spawned, so **nothing is
fetched or compiled**. Every other runtime genuinely installs. A user is told a
package was added when only a text edit happened.

**Options:** (a) spawn the detected tool (`mvn dependency:get` /
`gradle dependencies`) so the claim is true; (b) if intentionally deferred,
print an explicit `manifest updated — run 'mvn install' to fetch` and do not
mark it installed in `ven.toml`.

**Verify:** in a Maven project, `ven add <coord>` either leaves the artifact in
the local repository (`~/.m2/repository/...`) or prints the deferred notice;
`ven status` must not claim an installed package that is absent.

---

## Phase 2 — P1 latency (the two numbers users feel)

### P2-1 Kill the 300 ms project-`cd` spawn (F2) → M1 ≤ 15 ms

**Why it's hard:** activation must reflect `ven.toml` contents, so the hook
cannot simply skip the spawn — it must know the answer cheaply.

**Options, in order of preference:**

1. **Signature-keyed activation cache.** `ven shell activate` writes the rendered
   exports next to the manifest (or into `~/.ven/cache/`) keyed by the
   file's mtime+path signature the hook *already* computes. The hook reads the
   cache file (sub-millisecond) and only spawns ven when the signature is
   unknown. Invalidation is exact: any `ven.toml` edit changes the signature.
2. **Keep-alive helper.** A tiny long-lived process (`ven-shell-serve`) over a
   local socket that answers activation requests, eliminating per-`cd` process
   creation; hook falls back to a spawn if the helper is absent.
3. **Do more in-shell.** Move pure path composition into the hook (already
   true for the fast path) and have ven emit a declarative manifest the hook can
   apply without `Invoke-Expression`.

**Verify:** measure the slow path with the report's method before/after; assert
no activation when `ven.toml` is edited (signature changes) and that
`ven deactivate` still restores baseline state. Cover all four hook shells —
`bash`/`zsh`/`fish`/`powershell` each have their own fast-path test today
(`shell/mod.rs:705+`), extend them for the cache branch.

### P2-2 Cut startup from 256 ms toward 150 ms (F3) → M3

**Order of investigation (measure each, keep only what pays):**

1. Split the 170 ms above floor into *image scan* vs *program logic* —
   re-run with the Defender exclusion from `performance.md` applied; the delta
   is the scan.
2. Defer heavyweight initialisation (tokio multi-thread runtime, reqwest root
   pool, SQLite `intelligence.db` open) until a command actually needs it —
   `--version`/`path`/`status` should not pay for the network stack.
3. Re-evaluate default `gui` feature (`Cargo.toml:146`) for release artifacts so
   no GUI deps are compiled or linked into the CLI binary.
4. Code signing remains the definitive fix (already documented in
   `performance.md`).

**Verification:** M3 with N=5 warm; confirm no command regressed and that
`ven add`/`ven check` still initialise their subsystems (behaviour tests).

### P2-3 Reduce `ven doctor` worst case (F10) → M4

Profile which sub-check dominates the 2.9 s sample (likely multi-install
enumeration and PATH probing), cache immutable results, and parallelise the
independent probes.

### P2-4 Label stub-simulated analysis honestly (S1)

**Problem.** 7 of 9 ecosystems route to `GenericStubAdapter`
(`src/intelligence/adapters/mod.rs`, stubs at `:257`, `:268`, `:277`, `:287`,
`:298`, `:319`, `:331`), which reports *compatible* unconditionally while
carrying the notice at `:187`. Once P1-4 renders warnings, the user finally sees
"best-effort stub" — but the report and `--json` should also carry a structured
flag (e.g. `"analysis": "stub"`) so tooling can distinguish *verified clean*
from *not analysed*.

**Target metric:** a consumer reading `ven add --json` can always tell whether
a real dependency simulation ran.

**Verify:** JSON schema test per ecosystem — npm/Bun report `"real"`, the other
seven report `"stub"`; no ecosystem may report `compatible: true` with
`analysis: "stub"` and no warning.

**Sequencing note:** this is the structural counterpart to the storage work
below — a real adapter for Python/Go is the same investment class as a store.

---

## Phase 3 — P2 storage & throughput (where pnpm/uv currently win)

### P3-1 Content-addressable store for package installs (F8) → M9

Evaluate two routes:

- **Shell out to a CAS-aware client** where available (pnpm for npm-family,
  uv for Python) — smallest change, inherits their linking and parallelism.
- **ven-owned store** under `~/.ven/store` with hardlinks/reflinks per package,
  shared across projects — larger change, works for every ecosystem.

**Measure:** second-project marginal disk cost and install time; target ≈ 0 MB
and ≤ 1.5 s for the express tree (baseline: full copy, 1439 ms pnpm / 3772 ms npm).

### P3-2 Cut gate overhead (F6-adjacent)

Isolate OSV + latest-version resolution cost in `ven add`; cache OSV responses
by (name, version) in `intelligence.db` with a short TTL, and batch queries.
Target: gate overhead ≤ 100 ms warm, while keeping the security decision
unchanged (a cache miss must still query).

### P3-3 First-run UX for `ven sync` (F7)

Keep the lockfile contract, but when `ven.toml` has `[packages]` and no
`ven.lock` exists, either auto-generate with a clear notice or print a single
copy-pasteable `ven lock && ven sync`. Documented behaviour, so this is a
message-level change, not a semantics change.

---

## Phase 4 — P3 hygiene

- **`ven doctor` warnings** for the environment findings recorded in the report:
  intercepted TLS, keyword-blocked registries, missing toolchain.
- **Extend the metric set** (report's "Not yet measured"): peak RSS, parallel
  download throughput, 50+ dependency trees, `check`/`graph`/`why` latency,
  cold-boot numbers.
- **Reconcile `docs/performance.md`** with fresh numbers after Phase 2 lands;
  its current claims (~220–300 ms warm, fast-path fix) are consistent with this
  baseline, so no edit is needed until targets move.
- **Correct stale planning docs (S7):** `docs/worked_dev_docs/BUG_FIX_NEEDED.md`
  claims duplicate `VenConfig`/`RuntimeConfig` definitions "will cause
  compilation errors" — those duplicates are not present in this tree, and
  `cargo test` (once P0-1 lands) is the empirical way to confirm and retire it.

---

## Suggested sequencing

```
P0-1 (toolchain) ─┬─> P1-1 python fix ─┬─> P2-1 cd-hook cache ─> P3-1 store
P0-2 (bench suite)┘   P1-2 HTTPS       │
                    P1-3 PHP closed    └> P2-2 startup
                    P1-4 warnings          P2-4 stub provenance
                    P1-5 java honesty
```

P0 must land first: without `cargo test` every later change ships unverified.
P1 items are small, independent, and each closes a concrete defect. P2 is where
the headline numbers move. P3 is structural and should follow measurements taken
*after* P2, since startup and hook changes may alter the storage picture.

---

## Out of scope / explicitly deferred

- Pushing the existing `security/fix-ruby-7z-path-traversal` branch — the user
  has not asked for it.
- Changing `docs/performance.md` claims (currently consistent with measured
  data).
- Installing system toolchains without an explicit go-ahead (P0-1 options are
  listed for a decision, not executed).
