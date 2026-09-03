# ven set global

Make an **already installed** runtime available from every directory and
every new shell by persisting its `bin` dir on the **User PATH** — no
admin rights, nothing downloaded.

This complements `ven use` / shell hooks: those activate a runtime for the
*current project* (via `ven.toml`), while `ven set global` activates a
runtime for the *whole user account* (system-wide PATH, user scope). It's
the right tool when you just want `node`, `python`, or `cargo` on PATH
everywhere without admin access to `C:\Program Files`.

## How it persists

| Platform | Where the entry goes |
|----------|----------------------|
| Windows  | **User PATH** in the registry (`HKCU\Environment\Path`), updated via PowerShell `[Environment]::SetEnvironmentVariable('Path', …, 'User')` + a `WM_SETTINGCHANGE` broadcast so already-open shells pick it up |
| Linux / macOS | A fenced block in your rc files (`.bashrc`, `.zshrc`, `.profile`, `config.fish`): `# >>> ven global PATH >>>` … `# <<< ven global PATH <<<`, one `export PATH="…:$PATH"` line per runtime |

Both are idempotent: re-setting the same version is a no-op, and entries
are never duplicated. Setting a **different version of the same language**
replaces the old entry — one global version per language, and the
most recently set one wins.

The entry is **prepended**, so the ven-managed runtime wins over other
User-scope installs of the same tools (e.g. a corporate Node.js in
`%USERPROFILE%\NodeJS`). Note that on Windows the **Machine PATH is
ordered before the User PATH**: if the same tool is installed at machine
scope, that copy still resolves first and must be removed or reordered
for the ven global to take effect.

`ven set global` (no arguments) lists **only ven-managed entries** —
anything outside `$VEN_HOME` that happens to sit on the User PATH is not
shown.

> **Versions are restricted to what's already installed** under
> `$VEN_HOME` (`ven list` shows them). This command never fetches or
> downloads anything — it only rewrites PATH.

## Usage

```bash
ven set global                    # list current global entries
ven set global node               # pick an installed Node version interactively
ven set global node 20            # highest installed Node 20.x (resolved against installed)
ven set global python 3.12.7      # exact installed version
ven set global rust --unset       # remove every global PATH entry for Rust
ven set global node 20 --unset    # remove only that version's entry
ven set global --json             # machine-readable listing
```

### `ven set global <language>`

With a language but no version, ven picks the version for you:

- exactly one version installed → uses it directly;
- several installed → interactive picker (same style as `ven delete`);
- none installed → error, with a hint to run `ven install <language>`.

### `ven set global <language> <version>`

The spec is resolved **against installed versions only** using the same
per-language resolvers as `ven status` — so `node 20` means "the highest
installed 20.x", `python 3.12` means "highest installed 3.12.x", and an
exact version must already be installed.

### `ven set global <language> --unset`

Removes the language's global PATH entry (all installed versions by
default, or just the one given). If nothing was set, it's a no-op.

## Example

```bash
$ ven set global node 20

  [OK] node 20.11.0 is now globally available (User PATH).
  [PATH] C:\Users\me\.ven\node\20.11.0\bin

  To use it in THIS terminal right now, run:

    $env:Path = "C:\Users\me\.ven\node\20.11.0\bin" + ';' + $env:Path

  New terminals will pick it up automatically (restart your shell if needed).
```

The printed one-liner applies the change to the **current** terminal
(useful immediately); every future shell gets it from the persisted
User PATH automatically.

## Interaction with ven-managed installs

- `ven set global` entries point inside `$VEN_HOME`, so `ven delete` of
  that version leaves a stale PATH entry until you `--unset` it (or set a
  different version).
- `ven uninstall` cleans up: it strips the ven bin dir **and** every
  per-runtime global entry from the User PATH (Windows), and the rc-file
  block scrubber removes `# >>> ven global PATH >>>` blocks (Unix).

## JSON output

All forms accept `--json`:

```json
{
  "action": "set",
  "language": "node",
  "version": "20.11.0",
  "bin": "C:\\Users\\me\\.ven\\node\\20.11.0\\bin",
  "scope": "user"
}
```

`ven set global --json` lists `{ "global": [ { "path", "language",
"version" }, … ] }`.