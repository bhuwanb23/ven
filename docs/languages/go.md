# Go in ven

Go installs are pulled directly from **go.dev/dl** as the official archive, no checksums step required (Go's release archives are content-hashed by their immutable URLs and re-installs replace the whole tree).

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.go` |
| Install dir           | `~/.ven/go/<version>/` |
| Source                | `https://go.dev/dl/go<X.Y.Z>.<os>-<arch>.{zip,tar.gz}` |
| Release index         | `https://go.dev/dl/?mode=json&include=all` (only `stable: true` releases kept) |
| Architectures         | Windows / Linux / macOS · `amd64`, `arm64` |
| Package manager       | `go get` (via `go mod`) |
| Plugin                | `src/plugins/go.rs` |
| Downloader            | `src/core/go_install.rs` |

## Install

```bash
ven install go 1.22.3          # exact
ven install go 1.22            # latest 1.22.x
ven install go 1               # latest 1.x.y
ven install go latest          # newest stable
ven install go go1.22.3        # the "go" prefix is stripped automatically
ven install go                 # interactive picker
```

### Extraction

The archive ships everything under a top-level `go/` directory. The installer strips that prefix so files land directly in `~/.ven/go/<version>/`:

```
~/.ven/go/1.22.3/
├── bin/go         (or go.exe on Windows)
├── bin/gofmt
├── pkg/
├── src/
├── lib/
├── api/
└── …
```

If `bin/go` doesn't exist after extract, the install fails loudly.

## Activation

```toml
[runtime]
go = "1.22"
```

When active:

| Variable           | Value                                                |
|--------------------|------------------------------------------------------|
| `PATH` (prepended) | `~/.ven/go/<v>/bin`                                  |
| `VEN_GO_VERSION`   | Resolved version (e.g. `1.22.3`)                     |
| `GOROOT`           | `~/.ven/go/<v>` (parent of `bin`)                    |
| `GOPATH`           | `~/go` — the conventional default (per-user, not per-project) |

> **Note on `GOPATH`:** ven sets `GOPATH` to `$HOME/go` rather than something project-local. That matches the Go community default and keeps your module cache shared across projects. Override with `[env].GOPATH = "..."` in `ven.toml` if you need a sandbox.

## Packages — `go get`

When Go is the primary runtime, `ven add` runs the native Go workflow:

```bash
ven add github.com/gorilla/mux           # go get github.com/gorilla/mux
ven add github.com/gorilla/mux@v1.8.1    # go get <pkg>@<version>
```

Before the first add, ven runs `go mod init <folder-name>` if `go.mod` is missing. The resolved pin is then written into `[packages]` in `ven.toml` (the version stripped of the leading `@`).

`ven upgrade` and `ven remove` are not wired to Go's tooling yet — use `go get -u` / `go mod tidy` directly inside the activated shell.

There is no dependency-intelligence adapter for Go; the simulation layer returns the deterministic stub result.

### Configuration example

```toml
[runtime]
go = "1.22"

[packages]
"github.com/gorilla/mux" = "@v1.8.1"
"github.com/spf13/cobra" = "@latest"

[env]
GOFLAGS = "-mod=readonly"
```

## Common errors

| Symptom                                                                 | Cause / fix                                                              |
|-------------------------------------------------------------------------|---------------------------------------------------------------------------|
| `Go <v> is not installed. Run: ven install go <v>`                      | The pin in `ven.toml` doesn't match any folder under `~/.ven/go/`.        |
| `Cannot reach go.dev`                                                   | Network / proxy issue. Verify with `curl https://go.dev/dl/?mode=json`.  |
| `Failed to initialize go.mod (go mod init <name>)`                      | `go` not on PATH yet — activation may not have run. Open a new shell.    |
| `GOPATH` collisions with other Go installs                              | Set `[env].GOPATH = "..."` per project to isolate the module cache.       |
