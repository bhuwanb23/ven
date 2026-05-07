# Shell integration

## `ven setup`

Runs platform detection and writes hook snippets for supported shells (bash, zsh, fish, PowerShell). Goals:

- **`ven-use`** / **`ven-deactivate`** helpers so entering a directory applies `ven.toml` automatically.
- Optional profile snippets so manual PATH edits stay minimal.

After changing shells or reinstalling ven, re-run **`ven setup`** and follow printed instructions (source profile / restart terminal).

## `ven use` vs hooks

- **`ven use`** prints assignments intended for **evaluation in the current shell process**.  
  - POSIX: `eval "$(ven use)"`  
  - PowerShell: follow the stderr “hint” line printed by ven (Invoke-Expression pattern), or rely on **`ven-use`** after hooks.
- **`ven deactivate`** reverses the overlay **for that shell session** (based on variables ven exported).

## `ven shell *`

Low-level commands exist for hook scripts (`ven shell activate`, `ven shell deactivate`, `ven shell hook …`). Normal users should prefer **`ven setup`** + **`ven-use`**.

## Launcher

To spawn a **new** terminal with env applied without touching the current session, use **`ven-launcher`** (see [ven-launcher.md](ven-launcher.md)).
