# Install scripts

`scripts/install.ps1` (Windows) and `scripts/install.sh` (Linux / macOS) install `ven` directly from a GitHub release. They are thin orchestrators around [`ven-setup`](cmds/ven-setup.md): they detect the platform, fetch the release JSON, pick the right asset, optionally verify SHA256, and then either **delegate** to the self-contained `ven-setup` binary or **replicate** the install in shell when only raw-binary archives are present.

## One-liners

### Windows (PowerShell 5.1+)

```powershell
# user install, interactive prompt if a TTY is attached
irm https://get.ven.sh/install.ps1 | iex

# explicit modes via env var
$env:VEN_INSTALL_MODE='user';   irm https://get.ven.sh/install.ps1 | iex
$env:VEN_INSTALL_MODE='system'; irm https://get.ven.sh/install.ps1 | iex

# explicit modes via params (download-then-eval form)
& ([scriptblock]::Create((irm https://get.ven.sh/install.ps1))) -Mode system -Version v0.1.0
```

### Linux / macOS

```sh
# user install
curl -fsSL https://get.ven.sh/install.sh | sh

# explicit modes via flag (note the `--` separator after `sh -s`)
curl -fsSL https://get.ven.sh/install.sh | sh -s -- --mode user
sudo VEN_INSTALL_MODE=system bash -c "curl -fsSL https://get.ven.sh/install.sh | sh -s -- --mode system"

# explicit modes via env var
VEN_INSTALL_MODE=user curl -fsSL https://get.ven.sh/install.sh | sh
```

### Interim hosting

`get.ven.sh` is a placeholder until the domain is wired up. Use the raw GitHub URL in the meantime:

```powershell
irm https://raw.githubusercontent.com/yourorg/ven/main/scripts/install.ps1 | iex
```

```sh
curl -fsSL https://raw.githubusercontent.com/yourorg/ven/main/scripts/install.sh | sh
```

## Config surface

Both scripts read the same logical settings via env vars; the PowerShell one additionally exposes them as named `param()` flags, and the shell one as `--flag` arguments.

| Env var               | PowerShell `param` | Shell flag           | Default       | Purpose |
|-----------------------|--------------------|----------------------|---------------|---------|
| `VEN_INSTALL_MODE`    | `-Mode`            | `--mode <user/system>` | `user`        | Install scope. `user` = no admin / sudo. `system` = UAC (Windows) or `sudo` (Unix). |
| `VEN_VERSION`         | `-Version`         | `--version <tag>`    | `latest`      | GitHub release tag (e.g. `v0.1.0`) or `latest`. |
| `VEN_REPO`            | `-Repo`            | `--repo owner/name`  | `yourorg/ven` | GitHub `owner/repo` slug. Override for forks. |
| `VEN_NO_VERIFY`       | `-NoVerify`        | `--no-verify`        | `false`       | Skip SHA256 verification. |
| `VEN_DRY_RUN`         | `-DryRun`          | `--dry-run`          | `false`       | Print every step without touching the system. Download still happens during the dry-run preflight check; no writes. |
| `VEN_FORCE_REPLICATE` | `-ForceReplicate`  | `--force-replicate`  | `false`       | Skip the `ven-setup-*` asset even when present; use the raw-zip / tarball path. Useful for debugging the Replicate code path. |
| `VEN_DOCS_URL`        | -                  | -                    | `https://docs.ven.sh` | URL printed in the final completion banner. Override for forks. |
| `GITHUB_TOKEN`        | -                  | -                    | -             | Optional. Added to the API request as a Bearer token to avoid GitHub rate limits. |

### Mode selection precedence

1. Explicit `--mode` (shell) or `-Mode` (PowerShell).
2. `VEN_INSTALL_MODE` env var.
3. Interactive `1 / 2` prompt **only** when stdin is a TTY.
4. Default `user` for non-TTY (piped) invocations.

System mode on Unix has no UAC equivalent: the script refuses to proceed without root and prints the exact `sudo` re-run hint. System mode on Windows is fully self-handled by `ven-setup`'s own UAC relaunch in the Delegate path; the Replicate path requires the script itself to already be running elevated.

## Flow

```mermaid
flowchart TD
  start[user runs irm or curl] --> detect[detect arch + os + elevation + tty]
  detect --> mode{pick mode}
  mode --> fetch[GitHub releases API<br/>repos owner ven releases latest]
  fetch --> pick{asset for arch}
  pick -->|ven-setup-os-arch present| delegate[Delegate path<br/>download ven-setup, run --mode m --no-input]
  pick -->|only ven-os-arch zip or tarball| replicate[Replicate path<br/>extract + PATH + hooks + verify in shell]
  delegate --> verify[run ven --version with merged PATH]
  replicate --> verify
  verify --> cleanup[delete temp]
  cleanup --> done[print "Open a new terminal"]
```

The Delegate path is **always preferred** when the appropriate `ven-setup-*` asset is in the release because `ven-setup` is already the canonical install implementation; the install scripts then become a thin "fetch the right binary" wrapper. The Replicate path mirrors [`src/bin/setup/windows.rs`](../src/bin/setup/windows.rs) and [`src/bin/setup/unix.rs`](../src/bin/setup/unix.rs) closely enough that bug fixes can be ported in either direction.

## Release asset naming contract

The scripts look up assets by exact filename. Any release workflow that produces `ven` artifacts MUST publish these names (a future `.github/workflows/release.yml` is out of scope for this change but bound by the same contract):

### Delegate-path binaries (preferred)

| Platform               | Asset name                       |
|------------------------|----------------------------------|
| Windows x64            | `ven-setup-windows-x64.exe`      |
| Windows arm64          | `ven-setup-windows-arm64.exe`    |
| Linux x64              | `ven-setup-linux-x64`            |
| Linux arm64            | `ven-setup-linux-arm64`          |
| macOS x64 (Intel)      | `ven-setup-macos-x64`            |
| macOS arm64 (Apple Si.)| `ven-setup-macos-arm64`          |

Each `ven-setup-*` binary is the **self-contained installer** with `ven` and `ven-launcher` embedded via `build.rs` + `include_bytes!`. Size is roughly the sum of the two embedded binaries plus ~200 KB of installer logic.

### Replicate-path fallback archives

| Platform               | Asset name                       | Contents                                  |
|------------------------|----------------------------------|-------------------------------------------|
| Windows x64            | `ven-windows-x64.zip`            | `ven.exe`, `ven-launcher.exe`             |
| Windows arm64          | `ven-windows-arm64.zip`          | `ven.exe`, `ven-launcher.exe`             |
| Linux x64              | `ven-linux-x64.tar.gz`           | `ven`, `ven-launcher` (mode 0755)         |
| Linux arm64            | `ven-linux-arm64.tar.gz`         | `ven`, `ven-launcher`                     |
| macOS x64              | `ven-macos-x64.tar.gz`           | `ven`, `ven-launcher`                     |
| macOS arm64            | `ven-macos-arm64.tar.gz`         | `ven`, `ven-launcher`                     |

### Integrity (preferred: per-asset sidecar; fallback: manifest)

For each asset above, publish either a per-asset `.sha256` sidecar **or** include the asset in a top-level `SHA256SUMS` manifest. Both forms are accepted; sidecars take precedence.

| Form                       | Asset name                                | Contents                                         |
|----------------------------|-------------------------------------------|--------------------------------------------------|
| Per-asset sidecar (preferred) | `<asset>.sha256` (e.g. `ven-setup-linux-x64.sha256`) | First line is the sha256, optionally followed by two spaces and the filename. |
| Manifest (fallback)        | `SHA256SUMS`                              | One `<sha256>  <asset-filename>` line per asset. |

If neither is published, both scripts print `[skip]` for the verification step and continue. Set `VEN_NO_VERIFY=true` to skip verification explicitly even when integrity files are present.

## Security model

- HTTPS-only downloads. PowerShell forces TLS 1.2 on PS 5.1 (default is TLS 1.0, which GitHub rejects).
- SHA256 verify against `SHA256SUMS` (default on; gracefully skips when missing).
- No registry / rc-file writes happen in Delegate mode -- everything flows through the already-tested Rust installer.
- Replicate mode is `set -eu` / `$ErrorActionPreference='Stop'` so partial failures abort cleanly.
- `--dry-run` is honored end-to-end: downloads still happen so you can confirm the right asset is selected, but no writes are made and no child installer is invoked.
- The scripts never run any code from the release payload other than `ven-setup` (Delegate) or the binaries they install (`ven setup` invocation for shell hooks). They do not source / `iex` anything fetched.
- Optional `GITHUB_TOKEN` makes the API request authenticated (only the GitHub API endpoint sees the token; it is not forwarded to asset downloads).

## UX

Both scripts produce a banner, an indented `Detecting system...` block, an `Install mode` / `Install path` summary, then a sequence of right-aligned step lines that end in `[ok]`, `[skip]`, `[dry-run]`, or `[FAIL]`. A boxed completion banner closes the run.

```text
ven Installer
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Detecting system...
  OS:           macOS
  Architecture: arm64
  Shell:        zsh
  sudo / root:  No (sudo available)

Install mode: User (no admin)
Install path: /Users/you/.ven/bin

  Resolving release (yourorg/ven latest)...           [ok]
  Selecting asset...                                  [ok: Delegate]
  Downloading ven-setup-macos-arm64...                [ok]
    9.7 MB downloaded
  Verifying SHA256...                                 [ok]
  Delegating to ven-setup (user)...                   [ok]
  Verifying installation...                           [ok]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[OK] ven 0.1.0 installed successfully!

Open a NEW terminal (or `exec $SHELL -l`) and run:
  ven --version
  ven init

Documentation: https://docs.ven.sh
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

When a step fails the marker becomes `[FAIL]` and the captured `stdout` + `stderr` of that step is printed before the script exits.

## Implementation notes

### Delegate vs Replicate

The Delegate path is ~50 lines of useful logic in each script; everything else is detection, asset selection, hashing, and dry-run plumbing. When a `ven-setup-*` asset is in the release, `install.ps1` reduces to "download exe; `Start-Process -Wait`" and `install.sh` to "download; `chmod +x`; exec".

The Replicate path exists as a defensive fallback so that:

1. A release tagged before `ven-setup` landed can still be installed by current scripts.
2. CI pipelines that ship only the raw artifacts (no installer) still work.
3. The Replicate code path can be exercised in isolation (`-ForceReplicate` / `--force-replicate`) for debugging without rebuilding `ven-setup`.

Whenever [`src/bin/setup/windows.rs`](../src/bin/setup/windows.rs) or [`src/bin/setup/unix.rs`](../src/bin/setup/unix.rs) changes its PATH / rc-file / `/etc/profile.d` behavior, the corresponding `Update-VenPath` / `ensure_user_rc_path` / `ensure_etc_profile_d_path` functions in the install scripts should be updated to match. The block delimiters (`# >>> ven-setup PATH >>>` ... `# <<< ven-setup PATH <<<`) are intentionally identical so an install via either route can be cleanly uninstalled later.

### Why no jq / no PowerShell modules

Both scripts intentionally avoid external dependencies beyond the platform baseline:

- `install.ps1` uses only built-in cmdlets (`Invoke-WebRequest`, `Invoke-RestMethod`, `Get-FileHash`, `Expand-Archive`, `Start-Process`).
- `install.sh` uses only `curl`, `tar`, `awk`, `sed`, `mktemp`, `install`, `sha256sum` / `shasum`. It is `sh`-compatible (not bash-only) so it runs on Alpine, busybox, and macOS' default `/bin/sh`.

## Out of scope

- `.github/workflows/release.yml` -- producing the assets listed above is a separate concern. The naming contract above is the only coupling.
- `get.ven.sh` DNS and hosting -- use the raw GitHub URLs until the domain exists.
- `ven self-update` -- a future feature; for now re-run the install one-liner.
- Uninstall -- planned as `ven-setup --uninstall` (separate plan).
