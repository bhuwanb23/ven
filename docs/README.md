# `ven` documentation

This folder is the canonical reference for **CLI behavior**, **per-language plugin internals**, and **configuration**. Pair it with the top-level [README](../README.md) (marketing / quickstart) and `ven --help` / `ven <command> --help` (terse reference).

## Start here

| Topic                                          | File                                                       |
|------------------------------------------------|------------------------------------------------------------|
| Overview & quickstart                          | [`../README.md`](../README.md)                             |
| `ven.toml` schema (every key, every section)   | [`ven-toml.md`](ven-toml.md)                               |
| Every CLI command, one table                   | [`commands-reference.md`](commands-reference.md)           |
| Shell hooks · `ven setup` · `ven-use` flow     | [`shell-integration.md`](shell-integration.md)             |
| `ven-launcher` (spawn a terminal in a project) | [`ven-launcher.md`](ven-launcher.md)                       |

## Per-language deep dives

How each runtime is installed, where its files live, what activation exports, and how `ven add` / `ven remove` / `ven upgrade` behave.

| Runtime          | Doc                                                |
|------------------|----------------------------------------------------|
| Node.js          | [`languages/node.md`](languages/node.md)           |
| Python           | [`languages/python.md`](languages/python.md)       |
| Go               | [`languages/go.md`](languages/go.md)               |
| Rust             | [`languages/rust.md`](languages/rust.md)           |
| Java (JDK)       | [`languages/java.md`](languages/java.md)           |
| Deno             | [`languages/deno.md`](languages/deno.md)           |
| Bun              | [`languages/bun.md`](languages/bun.md)             |
| Ruby (MRI)       | [`languages/ruby.md`](languages/ruby.md)           |

The hub page [`languages.md`](languages.md) explains the `LanguagePlugin` trait, the on-disk layout under `~/.ven/`, and how to add support for a new runtime.

## Per-command pages

`docs/cmds/` has one page per CLI subcommand (`install`, `init`, `add`, `remove`, `upgrade`, `status`, `setup`, `shell`, `list`, `check-add`, `graph`, `lock`, `sync`, `resolve`).

Start with [`cmds/INDEX.md`](cmds/INDEX.md).
