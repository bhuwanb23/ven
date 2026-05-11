# Ruby in ven

Ruby is the most platform-divergent runtime in `ven`: on Windows it pulls **RubyInstaller2** `.7z` bundles, on macOS / Linux it pulls prebuilt **MRI** tarballs from the `ruby/ruby-builder` GitHub releases. Either way you get a fully self-contained `ruby` + `gem` + standard library tree.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.ruby` |
| Install dir           | `~/.ven/ruby/<X.Y.Z>/` |
| Windows source        | `https://github.com/oneclick/rubyinstaller2/releases/.../rubyinstaller-<X.Y.Z>-<build>-{x64,arm}.7z` |
| Unix source           | `https://github.com/ruby/ruby-builder/releases/tag/ruby-<X.Y.Z>` (per-OS tarball) |
| Architectures         | Windows: `x64`, `arm` · Linux: `ubuntu-22.04/24.04 x64+arm64` · macOS: `darwin x64+arm64` |
| Package manager       | `gem install` (`ven add` wires this) |
| Plugin                | `src/plugins/ruby.rs` |
| Downloader            | `src/core/ruby_install.rs` |
| Gem helpers           | `src/core/ruby_gems.rs` |

## Install

```bash
ven install ruby 3.4.2         # exact
ven install ruby 3.4           # latest 3.4.x
ven install ruby latest        # newest stable
ven install ruby               # interactive picker
```

### What the installer actually does

**Windows (RubyInstaller2):**

1. Scans up to 6 pages of `oneclick/rubyinstaller2` GitHub releases, filtering assets named `rubyinstaller-<X.Y.Z>-<build>-<arch>.7z`.
2. Keeps the **highest build number** per semver (e.g. `4.0.3-1` wins over `4.0.3-0` if both exist).
3. Downloads the `.7z` into `~/.ven/.cache/`.
4. Unpacks it with the in-process `sevenz-rust` crate into a temp dir, then relocates the inner `ruby-<…>/` folder into `~/.ven/ruby/<X.Y.Z>/`.

**macOS / Linux (`ruby/ruby-builder`):**

1. `GET /repos/ruby/ruby-builder/releases/tags/ruby-<X.Y.Z>`.
2. Picks the asset whose filename matches the host: `ubuntu-24.04-x64`, `ubuntu-22.04-arm64`, `darwin-x64`, `darwin-arm64`, etc. (precedence list per `platform_ruby_tarball_needles()`).
3. Downloads the `.tar.gz` and extracts it.
4. Relocates the inner `ruby-<…>/` folder into `~/.ven/ruby/<X.Y.Z>/`. Preserves symlinks on Unix.

Pre-release tags (`preview`, `rc`, `dev`) are filtered out of the listing on Unix.

### Layout after install

```
~/.ven/ruby/3.4.2/
├── bin/ruby       (or ruby.exe + ruby.cmd shims on Windows)
├── bin/gem
├── bin/bundle
├── bin/irb
├── include/
├── lib/
│   └── ruby/
│       ├── 3.4.0/                 (stdlib, ABI-versioned)
│       └── gems/3.4.0/            (GEM_HOME points here)
└── share/
```

## Activation

```toml
[runtime]
ruby = "3.4"
```

When active:

| Variable             | Value                                                          |
|----------------------|----------------------------------------------------------------|
| `PATH` (prepended)   | `~/.ven/ruby/<v>/bin`                                          |
| `VEN_RUBY_VERSION`   | Resolved version                                                |
| `GEM_HOME`           | Highest-numbered subdir of `~/.ven/ruby/<v>/lib/ruby/gems/`     |
| `GEM_PATH`           | Same as `GEM_HOME` — gems can't leak in from system Ruby        |

`ruby_gem_home_for_layout()` reads the actual on-disk layout to find the ABI directory (e.g. `lib/ruby/gems/3.4.0`) and exports that. This is what isolates each ven-Ruby's gems from the system gems.

## Packages — `gem install`

When Ruby is the primary runtime:

```bash
ven add rails                  # gem install --no-document rails
ven add rails@7.1.3            # gem install --no-document rails -v 7.1.3
ven add 'sidekiq@*'            # same as no version
ven remove sidekiq             # gem uninstall sidekiq -aIx (all versions, no prompt)
ven upgrade sidekiq            # preview: rubygems.org latest vs locally installed
ven upgrade sidekiq --apply    # gem install (latest); pin recorded as ">=<resolved>"
```

`ven` calls **plain `gem`** from `PATH` — meaning it relies on activation having set `GEM_HOME` first. If you run `ven add` outside an activated shell you'll install into whatever system Ruby PATH points at.

The pin written to `ven.toml` is `>=<installed-version>`, taken from `gem list -e ^<name>$` output. Override by passing `@<version>` to `ven add`.

### Bundler / Rails

`bundle install` works as usual once Ruby is active:

```bash
ven install ruby 3.4.2
ven init                # pick Ruby, version 3.4.2
echo 'source "https://rubygems.org"' > Gemfile
echo 'gem "sinatra"' >> Gemfile
bundle install          # installs into GEM_HOME = ~/.ven/ruby/3.4.2/lib/ruby/gems/3.4.0
```

Bundler honors `GEM_HOME`, so all gem state stays inside ven's directory tree.

### Configuration example

```toml
[runtime]
ruby = "3.4"

[packages]
sinatra = ">=4"
rake = "*"
pry = ">=0.14"

[env]
RUBYOPT = "-W:no-deprecated"
```

## Common errors

| Symptom                                                                    | Cause / fix                                                                |
|----------------------------------------------------------------------------|----------------------------------------------------------------------------|
| `No RubyInstaller2 build found for <v>` (Windows)                          | RI2 hasn't published a `.7z` for that exact semver. Try `latest` or a close patch. |
| `Ruby <v> not published on ruby/ruby-builder` (Unix)                       | Same idea — `ruby/ruby-builder` releases lag upstream MRI slightly.        |
| `No tarball assets for Ruby <v>` (Unix)                                    | Your OS/arch isn't in `platform_ruby_tarball_needles()` (Linux is currently ubuntu-22.04 / ubuntu-24.04 only — other distros aren't published). |
| `Unsupported Windows architecture for RubyInstaller2`                      | RI2 supports `x64` and `arm` only.                                         |
| `gem install` works but `bundle install` complains about permissions       | Confirm `GEM_HOME` is inside `~/.ven/ruby/...` (run `ven status` / `gem env`). |
| `Unpack did not yield bin/ruby[.exe]`                                      | The upstream archive layout changed. File an issue with the URL.            |
