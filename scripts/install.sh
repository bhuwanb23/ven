#!/usr/bin/env sh
# shellcheck shell=sh
#
# One-liner installer for ven on Linux / macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh -s -- --mode system
#   VEN_INSTALL_MODE=system curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh
#
# Reinstall over an existing copy:
#   VEN_FORCE_INSTALL=true curl -fsSL https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh | sh
#   ./install.sh --force            # local invocation, same effect
#
# Mirrors src/bin/setup/unix.rs. Keep the rc-file / /etc/profile.d logic here
# in sync with the Rust installer's ensure_user_rc_path / ensure_etc_profile_d_path.

set -eu

# ---------------------------------------------------------------------------
# Config (flags > env vars > defaults)
# ---------------------------------------------------------------------------

ven_mode="${VEN_INSTALL_MODE:-}"
ven_version="${VEN_VERSION:-latest}"
ven_repo="${VEN_REPO:-bhuwanb23/ven}"
ven_no_verify="${VEN_NO_VERIFY:-false}"
ven_dry_run="${VEN_DRY_RUN:-false}"
ven_force_replicate="${VEN_FORCE_REPLICATE:-false}"
ven_force_install="${VEN_FORCE_INSTALL:-false}"
ven_docs_url="${VEN_DOCS_URL:-https://bhuwanb23.github.io/ven/docs}"

while [ $# -gt 0 ]; do
    case "$1" in
        --mode)             ven_mode="$2"; shift 2 ;;
        --mode=*)           ven_mode="${1#*=}"; shift ;;
        --version)          ven_version="$2"; shift 2 ;;
        --version=*)        ven_version="${1#*=}"; shift ;;
        --repo)             ven_repo="$2"; shift 2 ;;
        --repo=*)           ven_repo="${1#*=}"; shift ;;
        --no-verify)        ven_no_verify="true"; shift ;;
        --dry-run)          ven_dry_run="true"; shift ;;
        --force-replicate)  ven_force_replicate="true"; shift ;;
        --force)            ven_force_install="true"; shift ;;
        -h|--help)
            sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            printf 'install.sh: unknown argument: %s\n' "$1" >&2
            exit 2
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Tiny helpers
# ---------------------------------------------------------------------------

# Box ruler. Width 56 chars to fit a typical 80-col terminal with margin.
LINE='━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'

err()         { printf '\nerror: %s\n' "$*" >&2; exit 1; }
say()         { printf '%s\n' "$*"; }
require_cmd() { command -v "$1" >/dev/null 2>&1 || err "missing required command: $1"; }

# step_begin "label"  -> prints "  label...                          " (no \n)
# step_done [marker]  -> prints " [ok]\n" by default
# step_skip           -> "[skip]"
# step_dry            -> "[dry-run]"
# step_fail           -> "[FAIL]"   (caller is expected to exit/err afterwards)
step_begin() { printf '  %-50s' "${1}..."; }
step_done()  { printf ' %s\n' "${1:-[ok]}"; }
step_skip()  { step_done '[skip]'; }
step_dry()   { step_done '[dry-run]'; }
step_fail()  { step_done '[FAIL]'; }

# Run a command silently; on failure print the captured output and exit 1.
run_step() {
    label="$1"; shift
    step_begin "$label"
    if [ "$ven_dry_run" = 'true' ]; then
        step_dry; return 0
    fi
    if "$@" >"$ven_log" 2>&1; then
        step_done
    else
        step_fail
        printf '\n----- step output -----\n' >&2
        cat "$ven_log" >&2 || :
        exit 1
    fi
}

# ---------------------------------------------------------------------------
# Banner
# ---------------------------------------------------------------------------

printf '\nven Installer\n'
printf '%s\n\n' "$LINE"

# ---------------------------------------------------------------------------
# Detection
# ---------------------------------------------------------------------------

require_cmd uname
require_cmd curl
require_cmd tar
require_cmd mktemp

uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "$uname_s" in
    Linux)   ven_os='linux';  os_human='Linux'  ;;
    Darwin)  ven_os='macos';  os_human='macOS'  ;;
    *)       err "unsupported OS: $uname_s (Linux and macOS only)" ;;
esac

case "$uname_m" in
    x86_64|amd64)   ven_arch='x64'   ;;
    aarch64|arm64)  ven_arch='arm64' ;;
    *)              err "unsupported arch: $uname_m" ;;
esac

if [ "$(id -u)" -eq 0 ]; then
    ven_root='true';  root_human='Yes (root)'
elif command -v sudo >/dev/null 2>&1; then
    ven_root='false'; root_human='No (sudo available)'
else
    ven_root='false'; root_human='No (sudo unavailable)'
fi

if [ -t 0 ] && [ -t 1 ]; then ven_tty='true'; else ven_tty='false'; fi

# Best-effort active shell: prefer $SHELL, fall back to ps -p $$ -o comm=.
shell_name="$(basename "${SHELL:-}")"
if [ -z "$shell_name" ]; then
    shell_name="$(ps -p "$$" -o comm= 2>/dev/null | tr -d '\n' || true)"
fi
[ -n "$shell_name" ] || shell_name='unknown'

say  'Detecting system...'
printf '  OS:           %s\n'  "$os_human"
printf '  Architecture: %s\n'  "$ven_arch"
printf '  Shell:        %s\n'  "$shell_name"
printf '  sudo / root:  %s\n'  "$root_human"
printf '\n'

# ---------------------------------------------------------------------------
# Mode selection
# ---------------------------------------------------------------------------

if [ -z "$ven_mode" ]; then
    if [ "$ven_tty" = 'false' ]; then
        ven_mode='user'
    elif [ "$ven_root" = 'false' ] && ! command -v sudo >/dev/null 2>&1; then
        # No way to escalate; user is the only real choice.
        ven_mode='user'
    else
        printf 'Select install mode:\n'
        printf '  [1] User Install (recommended) -- no sudo, only for you\n'
        printf '  [2] System Install            -- sudo required, all users on this machine\n'
        printf 'Choose [1/2]: '
        read -r choice
        case "$choice" in
            1) ven_mode='user' ;;
            2) ven_mode='system' ;;
            *) err "invalid selection: '$choice'. Set --mode or VEN_INSTALL_MODE explicitly." ;;
        esac
    fi
fi

case "$ven_mode" in
    user|system) ;;
    *) err "invalid mode '$ven_mode'. Expected 'user' or 'system'." ;;
esac

# Plain shell does not implement "ven-setup-style" self-elevation; tell the
# caller to re-run under sudo. This matches src/bin/setup/unix.rs.
if [ "$ven_mode" = 'system' ] && [ "$ven_root" != 'true' ] && [ "$ven_dry_run" != 'true' ]; then
    err "system install requires root. Re-run with: sudo VEN_INSTALL_MODE=system $0"
fi

if [ "$ven_mode" = 'system' ]; then
    install_dir='/usr/local/bin'
    mode_human='System (/usr/local/bin)'
else
    install_dir="$HOME/.ven/bin"
    mode_human='User (no admin)'
fi

printf 'Install mode: %s\n' "$mode_human"
printf 'Install path: %s\n\n' "$install_dir"

# ---------------------------------------------------------------------------
# Temp scratch (with trap cleanup)
# ---------------------------------------------------------------------------

ven_tmp="$(mktemp -d -t ven-install-XXXXXX)"
ven_log="$ven_tmp/step.log"
: >"$ven_log"
cleanup() {
    if [ "$ven_dry_run" != 'true' ] && [ -d "$ven_tmp" ]; then
        rm -rf "$ven_tmp"
    fi
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# GitHub release fetch + asset selection
# ---------------------------------------------------------------------------

if [ "$ven_version" = 'latest' ]; then
    api_url="https://api.github.com/repos/$ven_repo/releases/latest"
else
    api_url="https://api.github.com/repos/$ven_repo/releases/tags/$ven_version"
fi

curl_auth=''
if [ -n "${GITHUB_TOKEN:-}" ]; then
    curl_auth="-H 'Authorization: Bearer $GITHUB_TOKEN'"
fi

resolve_release() {
    # shellcheck disable=SC2090
    release_json="$(sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth '$api_url'")"
    tag_name="$(printf '%s' "$release_json" | sed -n 's/.*"tag_name"[^"]*"\([^"]*\)".*/\1/p' | head -n1)"
    [ -n "$tag_name" ] || return 1
    printf '%s\n' "$release_json" >"$ven_tmp/release.json"
    printf '%s\n' "$tag_name"     >"$ven_tmp/tag.txt"
}

step_begin "Resolving release ($ven_repo $ven_version)"
if resolve_release >/dev/null 2>"$ven_log"; then
    release_json="$(cat "$ven_tmp/release.json")"
    tag_name="$(cat "$ven_tmp/tag.txt")"
    step_done
else
    step_fail
    cat "$ven_log" >&2 || :
    err "could not fetch release JSON from $api_url"
fi

# ---------------------------------------------------------------------------
# Existing-install detection
#
# Both install modes leave a deterministic ven binary behind:
#   user    ~/.ven/bin/ven        (PATH block in ~/.bashrc / ~/.zshrc / ~/.profile)
#   system  /usr/local/bin/ven    (/etc/profile.d/ven.sh — system-wide)
#
# We don't trust PATH order — find every install on disk and report all of
# them, then compare the *target* (mode + resolved tag) against what's there.
# This is what catches the "I installed system v0.1.1, then ran the user
# install and got two ven binaries shadowing each other" failure mode.
# ---------------------------------------------------------------------------

# Normalise resolved tag ('v0.1.5' or '0.1.5') to the bare semver that
# `ven --version` prints, so equality comparisons line up.
target_ver="${tag_name#v}"

# Populate `existing_lines` with one "<mode> <ver> <path>" record per install
# found. Awk-friendly, easy to iterate twice without re-running probe.
existing_lines=''
probe_install() {
    probe_mode="$1"
    probe_path="$2"
    [ -x "$probe_path" ] || return 0
    probe_ver="$("$probe_path" --version 2>/dev/null | sed -n 's/^ven \([^ ]*\).*/\1/p' | head -n1)"
    [ -n "$probe_ver" ] || probe_ver='?'
    existing_lines="${existing_lines}${probe_mode} ${probe_ver} ${probe_path}
"
}
probe_install 'user'   "$HOME/.ven/bin/ven"
probe_install 'system' '/usr/local/bin/ven'

if [ -n "$existing_lines" ]; then
    printf '\nExisting installation(s) detected:\n'
    printf '%s' "$existing_lines" | while IFS=' ' read -r m v p; do
        [ -n "$m" ] && printf '  - %-6s ven %s  (%s)\n' "$m" "$v" "$p"
    done
    printf '\n'

    # Find the entry that competes with the target mode (the one we'd
    # overwrite). Use grep on a leading "<mode> " anchor so 'user' doesn't
    # match 'system' and vice versa.
    conflict_line="$(printf '%s' "$existing_lines" | grep -E "^${ven_mode} " | head -n1 || true)"
    other_line="$(printf '%s' "$existing_lines" | grep -Ev "^${ven_mode} " | head -n1 || true)"

    if [ -n "$conflict_line" ]; then
        conflict_ver="$(printf '%s' "$conflict_line" | awk '{print $2}')"
        if [ "$conflict_ver" = "$target_ver" ] && [ "$ven_force_install" != 'true' ]; then
            printf 'ven %s (%s) is already installed at this exact version. Nothing to do.\n' "$target_ver" "$ven_mode"
            printf 'Set VEN_FORCE_INSTALL=true (or pass --force) to reinstall over the top.\n'
            printf 'To remove it instead, see: https://bhuwanb23.github.io/ven/install#uninstall\n'
            [ "$ven_dry_run" = 'true' ] || exit 0
        fi
    fi

    if [ "$ven_force_install" != 'true' ]; then
        if [ "$ven_tty" = 'true' ]; then
            if [ -n "$conflict_line" ]; then
                conflict_ver="$(printf '%s' "$conflict_line" | awk '{print $2}')"
                printf 'Continue and replace ven %s -> %s (%s)? [Y/n] ' \
                    "$conflict_ver" "$target_ver" "$ven_mode"
            else
                other_mode="$(printf '%s' "$other_line" | awk '{print $1}')"
                printf 'Continue and install ven %s (%s) alongside the existing %s install? PATH precedence will pick whichever is listed first. [Y/n] ' \
                    "$target_ver" "$ven_mode" "$other_mode"
            fi
            read -r reply
            case "$reply" in
                [Nn]*) printf 'Aborted.\n'; [ "$ven_dry_run" = 'true' ] || exit 0 ;;
            esac
        else
            if [ -n "$conflict_line" ]; then
                conflict_ver="$(printf '%s' "$conflict_line" | awk '{print $2}')"
                printf 'Pipe-mode (no TTY): would replace ven %s -> %s (%s).\n' \
                    "$conflict_ver" "$target_ver" "$ven_mode"
            else
                other_mode="$(printf '%s' "$other_line" | awk '{print $1}')"
                printf 'Pipe-mode (no TTY): would install ven %s (%s) alongside the existing %s install.\n' \
                    "$target_ver" "$ven_mode" "$other_mode"
                printf '              PATH precedence will pick whichever is listed first.\n'
            fi
            printf 'Aborting to avoid surprises. Set VEN_FORCE_INSTALL=true to proceed,\n'
            printf 'or run the uninstall snippet first:\n'
            printf '  https://bhuwanb23.github.io/ven/install#uninstall\n'
            [ "$ven_dry_run" = 'true' ] || exit 0
        fi
    else
        say 'VEN_FORCE_INSTALL=true; proceeding anyway.'
    fi
    printf '\n'
fi

# GitHub returns minified single-line JSON. The asset filename is always the
# last path segment of its browser_download_url, so we grep for asset URLs
# and match the trailing segment instead of trying to parse JSON in sh.
find_asset_url() {
    asset_name="$1"
    printf '%s' "$release_json" \
        | grep -oE 'https://github\.com/[^"[:space:]]+/releases/download/[^"[:space:]]+/[^"[:space:]]+' \
        | awk -v name="$asset_name" -F/ '$NF == name { print; exit }'
}

setup_asset_name="ven-setup-${ven_os}-${ven_arch}"
tar_asset_name="ven-${ven_os}-${ven_arch}.tar.gz"

step_begin 'Selecting asset'
setup_url=''
if [ "$ven_force_replicate" != 'true' ]; then
    setup_url="$(find_asset_url "$setup_asset_name" || true)"
fi
tar_url="$(find_asset_url "$tar_asset_name" || true)"

if [ -n "$setup_url" ]; then
    use_delegate='true'
    asset_name="$setup_asset_name"
    asset_url="$setup_url"
    step_done "[ok: Delegate]"
elif [ -n "$tar_url" ]; then
    use_delegate='false'
    asset_name="$tar_asset_name"
    asset_url="$tar_url"
    step_done "[ok: Replicate]"
else
    step_fail
    err "release $tag_name has neither '$setup_asset_name' nor '$tar_asset_name'"
fi

# Per-asset .sha256 sidecar (preferred) or fall back to a SHA256SUMS manifest.
sha_sidecar_url="$(find_asset_url "${asset_name}.sha256" || true)"
sums_url="$(find_asset_url 'SHA256SUMS' || true)"

# ---------------------------------------------------------------------------
# Download
# ---------------------------------------------------------------------------

download_path="$ven_tmp/$asset_name"

human_size() {
    bytes="$1"
    if [ "$bytes" -ge 1048576 ]; then
        awk -v b="$bytes" 'BEGIN { printf "%.1f MB", b / 1048576 }'
    elif [ "$bytes" -ge 1024 ]; then
        awk -v b="$bytes" 'BEGIN { printf "%.1f KB", b / 1024 }'
    else
        printf '%d B' "$bytes"
    fi
}

do_download() {
    sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth -o '$download_path' '$asset_url'"
}

run_step "Downloading $asset_name" do_download
if [ "$ven_dry_run" != 'true' ]; then
    bytes="$(wc -c <"$download_path" | tr -d ' ')"
    printf '    %s downloaded\n' "$(human_size "$bytes")"
fi

# ---------------------------------------------------------------------------
# SHA256 verify
# ---------------------------------------------------------------------------

sha256_tool=''
if command -v sha256sum >/dev/null 2>&1; then sha256_tool='sha256sum'
elif command -v shasum  >/dev/null 2>&1; then sha256_tool='shasum -a 256'
fi

step_begin 'Verifying SHA256'
if [ "$ven_no_verify" = 'true' ]; then
    step_skip
elif [ "$ven_dry_run" = 'true' ]; then
    step_dry
elif [ -z "$sha256_tool" ]; then
    step_skip
    printf '    note: no sha256sum / shasum found on $PATH\n'
elif [ -n "$sha_sidecar_url" ]; then
    sha_path="$ven_tmp/${asset_name}.sha256"
    if sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth -o '$sha_path' '$sha_sidecar_url'" >"$ven_log" 2>&1; then
        # Sidecar may be either "<hash>  <name>" or just "<hash>".
        expected="$(awk 'NR==1 { print $1 }' "$sha_path")"
        actual="$($sha256_tool "$download_path" | awk '{print $1}')"
        if [ -n "$expected" ] && [ "$actual" = "$expected" ]; then
            step_done
        else
            step_fail
            err "SHA256 mismatch for $asset_name (sidecar): expected '$expected', got '$actual'"
        fi
    else
        step_fail
        cat "$ven_log" >&2 || :
        err "failed to download ${asset_name}.sha256"
    fi
elif [ -n "$sums_url" ]; then
    sums_path="$ven_tmp/SHA256SUMS"
    if sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth -o '$sums_path' '$sums_url'" >"$ven_log" 2>&1; then
        expected="$(awk -v n="$asset_name" '$2 == n || $2 == "*"n {print $1; exit}' "$sums_path")"
        if [ -z "$expected" ]; then
            step_fail
            err "SHA256SUMS did not contain an entry for $asset_name"
        fi
        actual="$($sha256_tool "$download_path" | awk '{print $1}')"
        if [ "$actual" = "$expected" ]; then
            step_done
        else
            step_fail
            err "SHA256 mismatch for $asset_name (manifest): expected '$expected', got '$actual'"
        fi
    else
        step_fail
        cat "$ven_log" >&2 || :
        err "failed to download SHA256SUMS"
    fi
else
    step_skip
    printf '    note: neither %s.sha256 nor SHA256SUMS published in this release\n' "$asset_name"
fi

# ---------------------------------------------------------------------------
# Install: Delegate vs Replicate
# ---------------------------------------------------------------------------

VEN_RC_BLOCK_START='# >>> ven-setup PATH >>>'
VEN_RC_BLOCK_END='# <<< ven-setup PATH <<<'

append_block_if_missing() {
    rc="$1"; block="$2"
    if grep -F "$VEN_RC_BLOCK_START" "$rc" >/dev/null 2>&1; then
        return 0
    fi
    if [ -s "$rc" ] && [ "$(tail -c1 "$rc"; echo x)" != "$(printf '\nx')" ]; then
        printf '\n' >> "$rc"
    fi
    printf '%s\n' "$block" >> "$rc"
}

ensure_user_rc_path() {
    install_dir_arg="$1"
    block="$(printf '%s\nexport PATH="%s:$PATH"\n%s' "$VEN_RC_BLOCK_START" "$install_dir_arg" "$VEN_RC_BLOCK_END")"
    wrote_any='false'
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [ -f "$rc" ]; then
            append_block_if_missing "$rc" "$block"
            wrote_any='true'
        fi
    done
    if [ "$wrote_any" = 'false' ]; then
        printf '%s\n' "$block" > "$HOME/.profile"
    fi
}

ensure_etc_profile_d_path() {
    install_dir_arg="$1"
    mkdir -p /etc/profile.d
    cat > /etc/profile.d/ven.sh <<EOF
#!/bin/sh
# Installed by ven-setup
case ":\$PATH:" in
  *":$install_dir_arg:"*) ;;
  *) export PATH="$install_dir_arg:\$PATH" ;;
esac
EOF
    chmod 0755 /etc/profile.d/ven.sh
}

# --- Delegate path ---------------------------------------------------------

do_delegate_setup() {
    chmod +x "$download_path"
    "$download_path" --mode "$ven_mode" --no-input
}

# --- Replicate path --------------------------------------------------------

do_extract() {
    mkdir -p "$ven_tmp/extract"
    tar -xzf "$download_path" -C "$ven_tmp/extract"
}

do_install_binaries() {
    mkdir -p "$install_dir"
    install -m 0755 "$ven_tmp/extract/ven"          "$install_dir/ven"
    install -m 0755 "$ven_tmp/extract/ven-launcher" "$install_dir/ven-launcher"
}

do_path_user()   { ensure_user_rc_path "$install_dir"; }
do_path_system() { ensure_etc_profile_d_path "$install_dir"; }
do_shell_hook()  { "$install_dir/ven" setup; }

# --- Dispatch --------------------------------------------------------------

if [ "$use_delegate" = 'true' ]; then
    run_step "Delegating to ven-setup ($ven_mode)" do_delegate_setup
else
    run_step "Extracting"                   do_extract
    run_step "Installing to $install_dir"   do_install_binaries
    if [ "$ven_mode" = 'system' ]; then
        run_step "Writing /etc/profile.d/ven.sh" do_path_system
    else
        run_step "Updating shell rc files (PATH)" do_path_user
        run_step "Installing shell hook (ven setup)" do_shell_hook
    fi
fi

# ---------------------------------------------------------------------------
# Verify
# ---------------------------------------------------------------------------

do_verify() {
    PATH="$install_dir:$PATH" sh -c 'ven --version' >"$ven_log.verify" 2>&1
    cp "$ven_log.verify" "$ven_log"
}
run_step 'Verifying installation' do_verify

ven_version_line=''
if [ "$ven_dry_run" != 'true' ]; then
    ven_version_line="$(head -n1 "$ven_log.verify" 2>/dev/null || echo '')"
fi

# ---------------------------------------------------------------------------
# Done banner
# ---------------------------------------------------------------------------

printf '\n%s\n' "$LINE"
if [ "$ven_dry_run" = 'true' ]; then
    printf '[OK] dry-run complete (release %s)\n' "$tag_name"
else
    if [ -n "$ven_version_line" ]; then
        printf '[OK] %s installed successfully!\n' "$ven_version_line"
    else
        printf '[OK] ven %s installed successfully!\n' "$tag_name"
    fi
fi
printf '\nOpen a NEW terminal (or `exec $SHELL -l`) and run:\n'
printf '  ven --version\n  ven init\n'
printf '\nDocumentation: %s\n' "$ven_docs_url"
printf '%s\n' "$LINE"
