#!/usr/bin/env sh
# SPDX-License-Identifier: MIT
# scripts/uninstall.sh — canonical POSIX fallback for `ven uninstall`.
#
# Synced with `scripts/uninstall.ps1` and the native Rust implementation in
# `src/core/uninstaller.rs`. The website's UNINSTALL.advanced.{macos,linux}
# snippets in `ven_website/src/content/site.js` are generated from the same
# shape.
#
# Why this exists:
#   1. The `ven` binary may be broken / missing / removed and can't
#      self-uninstall.
#   2. CI / sysadmin contexts want a single curlable script.
#   3. Some users prefer reading the shell version before trusting the
#      native command.
#
# Why one script for Linux + macOS:
#   macOS ships BSD `sed`, which REQUIRES an empty backup-extension arg
#   (`-i ''`) for in-place edits. Linux distros ship GNU sed, which
#   forbids the empty arg (`-i ''` becomes "rename foo to ''foo"). We
#   branch on `uname` once at the top and use the right form throughout.
#
# Idempotent: re-running after a partial uninstall converges to clean.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/uninstall.sh | sh
#   # ── OR after a successful install ──
#   "$HOME/.ven/bin/ven-uninstall"
#
# Flags (env vars, since we want to stay sh-pipe-friendly):
#   VEN_UNINSTALL_USER_ONLY=1     # skip the system layer
#   VEN_UNINSTALL_SYSTEM_ONLY=1   # skip the user layer
#   VEN_UNINSTALL_DRY_RUN=1       # print plan, change nothing

set -u

# ── sed -i variant detection ────────────────────────────────────────────────
case "$(uname -s 2>/dev/null)" in
    Darwin) SED_INPLACE_ARGS="-i ''" ;;
    *)      SED_INPLACE_ARGS='-i' ;;
esac

# Run sed in-place against $1 using the OS-correct flag. We have to use eval
# because BSD requires `-i ''` (two words) and GNU `-i` (one word), and
# POSIX `sh` doesn't let us pass an empty positional cleanly.
ven_sed_inplace() {
    _file="$1"; _script="$2"
    eval "sed $SED_INPLACE_ARGS '$_script' \"\$_file\" 2>/dev/null" || true
}

# ── Scope ────────────────────────────────────────────────────────────────────
VEN_UNINSTALL_USER_ONLY="${VEN_UNINSTALL_USER_ONLY:-}"
VEN_UNINSTALL_SYSTEM_ONLY="${VEN_UNINSTALL_SYSTEM_ONLY:-}"
VEN_UNINSTALL_DRY_RUN="${VEN_UNINSTALL_DRY_RUN:-}"

if [ -n "$VEN_UNINSTALL_USER_ONLY" ] && [ -n "$VEN_UNINSTALL_SYSTEM_ONLY" ]; then
    printf 'VEN_UNINSTALL_USER_ONLY and VEN_UNINSTALL_SYSTEM_ONLY are mutually exclusive.\n' >&2
    exit 1
fi

DRY_TAG=''
if [ -n "$VEN_UNINSTALL_DRY_RUN" ]; then
    DRY_TAG='[DRY-RUN] '
fi

# ── Helpers ──────────────────────────────────────────────────────────────────
say() { printf '%s%s\n' "$DRY_TAG" "$1"; }

# Strip a fenced `# >>> name >>> ... # <<< name <<<` block from $1.
# Falls back to a python one-liner when neither awk nor sed can handle
# multi-line ranges cleanly enough across BSD + GNU.
strip_block() {
    _file="$1"; _name="$2"
    [ -f "$_file" ] || return 0
    grep -F "# >>> $_name >>>" "$_file" >/dev/null 2>&1 || return 0
    if [ -n "$VEN_UNINSTALL_DRY_RUN" ]; then
        say "would strip '$_name' block from: $_file"
        return 0
    fi
    _tmp="${_file}.ven-uninstall.tmp"
    awk -v start="# >>> $_name >>>" -v end="# <<< $_name <<<" '
        $0 == start { skip = 1; next }
        $0 == end   { skip = 0; next }
        skip == 0   { print }
    ' "$_file" > "$_tmp" 2>/dev/null && mv "$_tmp" "$_file"
    say "stripped '$_name' block from: $_file"
}

# Strip the *unfenced* `ven shell hook` block from $1.
#
# `ven shell install` and `ven shell hook <shell> >> ~/.bashrc`-style setups
# append the hook to EOF with one of the head markers below and NO closing
# fence — strip_block can't handle that. So this trims from the earliest
# head-marker line to end-of-file.
#
# Safety cap: 16 KB. The hook is ~1–2 KB across all three shells; anything
# bigger almost certainly means user content after the hook, which we'd
# rather leave intact (with a warning) than nuke. Mirrors HOOK_TRIM_BUDGET
# in src/core/uninstaller.rs and the PowerShell fallback.
strip_hook_block() {
    _file="$1"
    [ -f "$_file" ] || return 0
    _start_line=$(awk '
        /^# ven shell hook - Auto-loads on terminal start$/ { print NR; exit }
        /^# ven shell hook \(bash\/zsh\)/                   { print NR; exit }
        /^# ven shell hook \(fish\)/                        { print NR; exit }
        /^# ven shell hook \(PowerShell\)/                  { print NR; exit }
    ' "$_file" 2>/dev/null)
    [ -n "$_start_line" ] || return 0
    # Eat exactly one preceding blank line if present — the installer
    # prefixes a leading blank line before the wrapper banner.
    if [ "$_start_line" -gt 1 ]; then
        _prev_line=$((_start_line - 1))
        _prev_content=$(sed -n "${_prev_line}p" "$_file" 2>/dev/null)
        if [ -z "$_prev_content" ]; then
            _start_line=$_prev_line
        fi
    fi
    # Tail of the file from the matched line — that's what we'd remove.
    _trim_bytes=$(tail -n "+${_start_line}" "$_file" 2>/dev/null | wc -c | tr -d ' ')
    if [ -n "$_trim_bytes" ] && [ "$_trim_bytes" -gt 16384 ]; then
        say "WARN: skipping hook scrub of $_file: would drop ${_trim_bytes} bytes (>16 KB cap). Edit the file by hand to clear the '# ven shell hook' block."
        return 0
    fi
    if [ -n "$VEN_UNINSTALL_DRY_RUN" ]; then
        say "would strip 'ven shell hook' block from: $_file (line ${_start_line} to EOF)"
        return 0
    fi
    _keep_lines=$((_start_line - 1))
    _tmp="${_file}.ven-uninstall.tmp"
    if [ "$_keep_lines" -gt 0 ]; then
        head -n "$_keep_lines" "$_file" > "$_tmp" 2>/dev/null
    else
        : > "$_tmp"
    fi
    mv "$_tmp" "$_file"
    say "stripped 'ven shell hook' block from: $_file"
}

say 'ven uninstall (POSIX fallback script)'
if [ -n "$VEN_UNINSTALL_DRY_RUN" ]; then
    say '[i] Nothing will be removed; this is a plan-only run.'
fi

# ── 1. User install ─────────────────────────────────────────────────────────
if [ -z "$VEN_UNINSTALL_SYSTEM_ONLY" ]; then
    # 1a. Install root.
    if [ -d "$HOME/.ven" ]; then
        if [ -z "$VEN_UNINSTALL_DRY_RUN" ]; then
            rm -rf "$HOME/.ven"
        fi
        say "Removed user install: $HOME/.ven"
    fi

    # 1b. Rc-file cleanup. Two-stage hook scrub: the fenced form (legacy
    #     / future) goes through strip_block, the unfenced form (what
    #     `ven shell install` actually writes today) goes through
    #     strip_hook_block. Either is a no-op when its marker isn't
    #     present, so running both is safe. Then the orphan-line fallback
    #     mops up legacy installs that never used markers at all.
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.zprofile" "$HOME/.bash_profile" "$HOME/.profile"; do
        if [ -f "$rc" ]; then
            strip_block      "$rc" 'ven env'
            strip_block      "$rc" 'ven-setup PATH'
            strip_block      "$rc" 'ven shell hook'
            strip_hook_block "$rc"
            if [ -z "$VEN_UNINSTALL_DRY_RUN" ]; then
                ven_sed_inplace "$rc" '/\.ven\/bin/d'
            fi
        fi
    done

    # 1c. fish config (different rc file, also a candidate target of
    #     `ven shell hook` and the persisted-env writer).
    fish_cfg="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
    if [ -f "$fish_cfg" ]; then
        strip_block      "$fish_cfg" 'ven env'
        strip_block      "$fish_cfg" 'ven-setup PATH'
        strip_block      "$fish_cfg" 'ven shell hook'
        strip_hook_block "$fish_cfg"
        if [ -z "$VEN_UNINSTALL_DRY_RUN" ]; then
            ven_sed_inplace "$fish_cfg" '/\.ven\/bin/d'
        fi
    fi

    # 1d. Pointer file (~/.config/ven/config.toml).
    pointer_dir="${XDG_CONFIG_HOME:-$HOME/.config}/ven"
    pointer_file="$pointer_dir/config.toml"
    if [ -f "$pointer_file" ]; then
        if [ -z "$VEN_UNINSTALL_DRY_RUN" ]; then
            rm -f "$pointer_file"
        fi
        say "Removed pointer file: $pointer_file"
    fi
    # Drop the dir if it's now empty.
    if [ -d "$pointer_dir" ] && [ -z "$(ls -A "$pointer_dir" 2>/dev/null)" ]; then
        if [ -z "$VEN_UNINSTALL_DRY_RUN" ]; then
            rmdir "$pointer_dir" 2>/dev/null || true
        fi
    fi
fi

# ── 2. System install ───────────────────────────────────────────────────────
if [ -z "$VEN_UNINSTALL_USER_ONLY" ]; then
    if [ -e /usr/local/bin/ven ] \
        || [ -e /usr/local/bin/ven-launcher ] \
        || [ -e /usr/local/bin/ven-setup ] \
        || [ -e /etc/profile.d/ven.sh ]; then

        if [ "$(id -u 2>/dev/null || echo 0)" = '0' ]; then
            SUDO=''
        else
            SUDO='sudo'
        fi

        if [ -n "$VEN_UNINSTALL_DRY_RUN" ]; then
            say 'would remove system install:'
            for p in /usr/local/bin/ven /usr/local/bin/ven-launcher /usr/local/bin/ven-setup /etc/profile.d/ven.sh; do
                [ -e "$p" ] && say "  $p"
            done
        else
            $SUDO rm -fv \
                /usr/local/bin/ven \
                /usr/local/bin/ven-launcher \
                /usr/local/bin/ven-setup \
                /etc/profile.d/ven.sh
        fi
    fi
fi

# Refresh the shell's executable cache so a stale `ven` lookup doesn't lie.
hash -r 2>/dev/null || true

printf '\n'
if [ -n "$VEN_UNINSTALL_DRY_RUN" ]; then
    say 'Dry-run finished — nothing was touched. Unset VEN_UNINSTALL_DRY_RUN to execute.'
else
    say 'Done. Open a NEW terminal so the cleaned PATH takes effect.'
fi
