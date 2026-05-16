# ven-launcher

`ven-launcher` is a companion binary shipped with this repo. It:

1. Takes an optional **project path** (directory or path to `ven.toml`).
2. Walks up from that root to find **`ven.toml`** (same rules as `ven use`).
3. Either **opens a new terminal** with the resolved runtime PATH/env, or prints env.

## Usage

```text
ven-launcher [PROJECT]
```

- **`PROJECT`** omitted: use current working directory as search root.
- **`--show-env`**: print resolved environment / PATH information instead of spawning a terminal.

## When to use it

- IDE shortcuts / desktop shortcuts that should always open a shell "inside" a project.
- Users who prefer not to `eval`/`ven-use` in an existing shell.
- Corporate / locked-down environments where modifying the system `PATH` or rc files is not allowed.
- Non-CLI users who want to double-click an icon and get a ven-ready terminal — see [Double-click shim](#double-click-shim) below.

For in-shell activation without a new window, use **`ven use`** or hooks after **`ven setup`**.

## Double-click shim

The discoverable portable bundle (`ven-launcher-{os}-{arch}.{zip|tar.gz}`) ships a tiny per-OS shim file next to `ven-launcher` so a non-CLI user (think: corporate teammates behind Zscaler, designers, students) can extract the zip and double-click their way into a ven-ready terminal — **no command-line typing required**.

| OS      | Shim filename          | What happens on double-click                                                                                                          |
|---------|------------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| Windows | `Start ven.cmd`        | Explorer runs the `.cmd` (no execution-policy or admin prompt), which calls `ven-launcher.exe`, which opens a fresh PowerShell with `ven` activated. |
| macOS   | `Start ven.command`    | Finder treats `.command` as a Terminal script (first launch may show a Gatekeeper warning — right-click → Open once). Calls `./ven-launcher`. |
| Linux   | `start-ven.sh`         | Most file managers offer "Run in Terminal" for executable scripts. Otherwise: `./start-ven.sh` in any terminal.                       |

The shims themselves are 3–7 lines each — just a `cd "$(dirname "$0")"` followed by an `exec` of `ven-launcher`. They make no network calls and require no special permissions. Power users can keep calling `./ven-launcher` directly; the shim is purely a discoverability layer.

### Behind Zscaler / corporate proxy

Most corporate web proxies (Zscaler, Symantec, Forcepoint) block `irm | iex` and `curl | sh` style one-liners because they look like script injection, but they do **not** block downloading a regular `.zip` from `github.com` over HTTPS. The portable bundle is designed for that constraint: download the zip through the browser, extract via Explorer / Finder, double-click the shim, get a working `ven`. Nothing in the path requires elevated permissions, hits a non-GitHub host, or touches the system `PATH`.

### Corporate cert trust (Zscaler / Netskope / Bluecoat) — v0.1.3+ required

Most enterprise web proxies don't just route traffic — they **MITM** every HTTPS connection so they can inspect the contents. They do this by issuing their own dynamic per-host certificate signed by a private root CA that an admin pre-installs into the Windows / macOS / Linux trust store. Your browser trusts it (browsers read the OS store), so `https://...` "just works" from Chrome / Edge / Firefox.

`ven` versions **≤ v0.1.2** used `rustls` with only the bundled Mozilla webpki-roots and ignored the OS store entirely. That meant `ven install python` (and any other download) failed with:

```text
Error: Cannot list Python releases: error sending request for url (https://www.python.org/ftp/python/)
```

…even though the URL opened fine in a browser on the same machine.

**Fix: upgrade to v0.1.3 or newer.** v0.1.3 enables reqwest's `rustls-tls-native-roots` feature, so `ven` merges the bundled Mozilla root pool with whatever roots the OS trusts — including the corporate intercept CA. No flags, no env vars, no `~/.config/ven/ca.pem` to maintain. The same binary works at home and behind Zscaler.

Verify the upgrade worked:

```pwsh
ven --version          # ven 0.1.3 (or newer)
ven install python     # should now reach python.org without "error sending request"
```

If you're still seeing TLS errors after upgrading to v0.1.3+:

1. Confirm the corporate root CA is actually installed in the OS trust store (`certmgr.msc` on Windows → Trusted Root Certification Authorities). If your browser shows a green padlock on `https://www.python.org`, it's there.
2. Make sure your admin hasn't blocked outbound access to `python.org` / `nodejs.org` / `go.dev` / `crates.io` etc. in the proxy. A cert problem looks like "error sending request"; a blocked-host problem looks like "403 Forbidden" or a hung connection — those are policy issues, not `ven` bugs.
3. If you're on a network with an explicit proxy (rare for Zscaler, common for older corp setups), set `HTTPS_PROXY=http://proxy.corp:8080` in the shell before running `ven`. reqwest reads `HTTPS_PROXY` / `HTTP_PROXY` / `NO_PROXY` automatically.

## Portable mode

`ven-launcher` resolves a single **storage root** (called `VEN_HOME`) on every run, then propagates that value to the spawned shell so every subsequent `ven` call inside it lands in the same place. Resolution is most-specific → least-specific:

| Order | Source                                                       | Notes                                                                 |
|------:|--------------------------------------------------------------|-----------------------------------------------------------------------|
|     1 | `$VEN_HOME` env var (if set & non-empty)                     | Explicit override; trumps everything else.                             |
|     2 | `$VEN_STORAGE_PATH` env var (if set & non-empty)             | Back-compat with the early modules that introduced this convention.    |
|     3 | `<dir-of-launcher-exe>/.ven` if that directory exists        | **Auto-portable.** Drop a `.ven/` next to `ven-launcher`, you're done. |
|     4 | `~/.ven`                                                     | Default for an installed ven.                                          |

### USB-stick / fully-portable workflow

Use the discoverable `ven-launcher-{os}-{arch}.{zip|tar.gz}` release asset (see [install-scripts.md](install-scripts.md#portable-launcher-bundle)) and:

```text
my-bundle/
├── ven                    # or ven.exe on Windows
├── ven-launcher           # or ven-launcher.exe on Windows
├── Start ven.command      # macOS shim (or Start ven.cmd / start-ven.sh)
├── README.txt
└── .ven/                  # create this folder once; everything ven downloads
                           # (runtimes, lockfile state, doc cache) lives here
```

`./ven-launcher --show-env` will then print `VEN_HOME should be: <bundle>/.ven` and the spawned shell will see the same value via the inherited `VEN_HOME` env var. No system `PATH` change, no rc-file edits, no `~/.ven` writes — the bundle is fully self-contained and movable.

To run the same bundle in shared mode instead (use the host's `~/.ven`), just delete the sibling `.ven/` folder; `ven-launcher` will fall back to tier 4.

### Explicit override

For containers, CI, or one-off testing where you want to point a single launcher invocation at an arbitrary directory:

```sh
VEN_HOME=/srv/ci-cache/ven ./ven-launcher
```

The launcher exports the resolved value to the child shell, so `ven status`, `ven install`, and `ven sync` inside that shell will all use `/srv/ci-cache/ven`.
