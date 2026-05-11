# Python in ven

`ven` ships an **embeddable** Python distribution (Windows-only at the binary level), then **bootstraps pip into it** so the resulting `~/.ven/python/<version>/python.exe` is usable for `pip install` and as a venv base.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.python` |
| Install dir           | `~/.ven/python/<version>/` |
| Source                | `https://www.python.org/ftp/python/<X.Y.Z>/python-<X.Y.Z>-embed-<arch>.zip` |
| Architectures         | Windows: `amd64`, `arm64`, `win32` (32-bit x86) |
| `latest_version()`    | First `3.x.y` entry in `https://www.python.org/ftp/python/` |
| Package manager       | `pip` (`<python> -m pip`) |
| Plugin                | `src/plugins/python.rs` |
| Downloader            | `src/core/python_install.rs` |
| Project venv helper   | `src/core/project_venv.rs` |

## Platform support

> **Important:** `ven install python` is implemented **for Windows only** today. The embeddable zip distribution is a Windows-specific artifact.
>
> On macOS / Linux, `runtime.python` in `ven.toml` is still honored — but it expects you to maintain a `./venv` (or legacy `./.venv`) yourself with `python3 -m venv venv`. Activation will detect that venv and prepend `venv/bin` to `PATH`. There is no `ven install python` step on Unix.

## Install (Windows)

```bash
ven install python 3.12.7      # exact (required for first install of a line)
ven install python 3.12        # latest 3.12.z published on python.org
ven install python latest      # newest 3.x.y on python.org
ven install python             # interactive picker
```

### What the installer actually does

1. **Download** the embeddable zip from `https://www.python.org/ftp/python/<X.Y.Z>/python-<X.Y.Z>-embed-<arch>.zip` into `~/.ven/.cache/`.
2. **Extract** into `~/.ven/python/<X.Y.Z>/` (any previous contents are removed first).
3. **Edit `python<XY>._pth`**: uncomment `# import site` so pip and site-packages work — the embeddable build ships with this disabled by default.
4. **Bootstrap pip** by running `python.exe -m ensurepip --upgrade`. If `ensurepip` is unavailable (some embed builds), ven downloads `https://bootstrap.pypa.io/get-pip.py` and runs it.
5. **Validate** `python --version` and `python -m pip --version` both succeed.

This is why you need a **full `X.Y.Z`** for the first install of a line — the embed download URL requires it. After that, `runtime.python = "3.12"` in `ven.toml` resolves against installed versions.

## Activation

```toml
[runtime]
python = "3.12"
```

Activation prefers, in order:

1. A **project-local venv** (`./venv` or legacy `./.venv`) if it has a valid `pyvenv.cfg`. Prepends `venv/Scripts` (Windows) / `venv/bin` (Unix), sets `VIRTUAL_ENV`.
2. Otherwise (Windows only), the ven-managed embed: prepends `~/.ven/python/<v>/Scripts` then `~/.ven/python/<v>/`.

Either way, these are exported:

| Variable             | Value                                                              |
|----------------------|--------------------------------------------------------------------|
| `VEN_PYTHON_VERSION` | Resolved version (from `pyvenv.cfg`'s `version =` line if available) |
| `VIRTUAL_ENV`        | Set to the project venv root when one is used                       |
| `PATH` (prepended)   | venv bin dir first, then ven-managed Python                         |

### Skipping the project venv

Set **`VEN_SKIP_PROJECT_VENV=1`** (the `ven deactivate` shell helper does this for you) to pause the auto-prepend of `./venv`. Running `ven-use` clears the flag.

## Project venvs

`ven init` creates `./venv` automatically when you pick Python:

- Tries `python -m venv --copies venv` first.
- If that fails (common with embeddable builds that **don't ship `venv`**), it `pip install`s **`virtualenv`** and uses `python -m virtualenv --copies venv`.
- Forces `include-system-site-packages = false` in `pyvenv.cfg`.
- Appends `venv/` (and legacy `.venv/`) to `.gitignore` if missing.

> The Windows embeddable distribution **does not include the stdlib `venv`** module — that's why ven has the `virtualenv` fallback. macOS/Linux Pythons normally have `venv` built in.

## Packages — pip

When Python is the primary runtime:

```bash
ven add fastapi               # python -m pip install fastapi
ven add 'pandas>=2.0,<3'      # full PEP 440 spec accepted
ven remove fastapi            # python -m pip uninstall -y fastapi
ven upgrade fastapi --apply   # pip install --upgrade
```

The Python binary used is resolved as:

1. `$VIRTUAL_ENV/Scripts/python.exe` (Windows) if the project venv is active.
2. `~/.ven/python/$VEN_PYTHON_VERSION/python.exe` if set.
3. `python` (Windows) / `python3` (Unix) from `PATH`.

`ven upgrade` reads `pip list --outdated --format=json` to decide which packages have newer releases, prints a preview, and only applies with `--apply`.

The dependency-intelligence layer uses a **stub adapter** for Python — `ven add` writes the pin into `ven.toml` and trusts pip's resolver. There is no pre-install graph simulation for PyPI yet.

### Configuration example

```toml
[runtime]
python = "3.12"

[packages]
fastapi = ">=0.110,<1.0"
uvicorn = "*"
pydantic = "==2.7.4"

[env]
PYTHONDONTWRITEBYTECODE = "1"

[venv]
auto_path = true   # default; prepend `./venv` when present
```

## Common errors

| Symptom                                                               | Cause / fix                                                                                  |
|-----------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| `Need full Python version for embeddable zip, e.g. 3.12.7`            | First install of a `3.x` line requires an exact patch. Run `ven install python 3.12.7`.       |
| `Python embed install is only implemented on Windows in this release` | Use system Python or pyenv on Unix; ven will still honor `runtime.python` if you make a venv. |
| `No module named venv`                                                | Expected on embeddable Python. `ven init` switches to `virtualenv` automatically.             |
| `Pip not installed` after `ven install`                               | The bootstrap step failed (network?). Run `<python.exe> -m ensurepip --upgrade` manually.     |
| `runtime.python` set but activation errors with "no `venv/`"          | Unix only: ven won't auto-create the venv. Run `python3 -m venv venv` once.                   |
