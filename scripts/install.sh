#!/usr/bin/env sh
# shellcheck shell=sh
#
# One-liner installer for ven on Linux / macOS.
#
# Usage:
#   curl -fsSL https://get.ven.sh/install.sh | sh
#   curl -fsSL https://get.ven.sh/install.sh | sh -s -- --mode system
#   VEN_INSTALL_MODE=system curl -fsSL https://get.ven.sh/install.sh | sh
#
# Mirrors src/bin/setup/unix.rs. Keep the rc-file / /etc/profile.d logic here
# in sync with the Rust installer's ensure_user_rc_path / ensure_etc_profile_d_path.

set -eu

# ---------------------------------------------------------------------------
# Config (flags > env vars > defaults)
# ---------------------------------------------------------------------------

ven_mode="${VEN_INSTALL_MODE:-}"
ven_version="${VEN_VERSION:-latest}"
ven_repo="${VEN_REPO:-yourorg/ven}"
ven_no_verify="${VEN_NO_VERIFY:-false}"
ven_dry_run="${VEN_DRY_RUN:-false}"
ven_force_replicate="${VEN_FORCE_REPLICATE:-false}"

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

err() { printf 'install.sh: error: %s\n' "$*" >&2; exit 1; }
info() { printf '%s\n' "$*"; }
require_cmd() { command -v "$1" >/dev/null 2>&1 || err "missing required command: $1"; }

# ---------------------------------------------------------------------------
# Banner + detection
# ---------------------------------------------------------------------------

printf '\n'
printf '  +-----------------------------------------+\n'
printf '  |  ven one-liner installer (Unix)         |\n'
printf '  +-----------------------------------------+\n'
printf '  repo:    %s\n' "$ven_repo"
printf '  version: %s\n' "$ven_version"
printf '  mode:    %s\n' "${ven_mode:-(prompt)}"
printf '  dry-run: %s\n' "$ven_dry_run"
printf '\n'

require_cmd uname
require_cmd curl
require_cmd tar
require_cmd mktemp

uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "$uname_s" in
    Linux)   ven_os='linux'  ;;
    Darwin)  ven_os='darwin' ;;
    *)       err "unsupported OS: $uname_s (Linux and Darwin only)" ;;
esac

case "$uname_m" in
    x86_64|amd64)        ven_arch='x64'   ;;
    aarch64|arm64)       ven_arch='arm64' ;;
    *)                   err "unsupported arch: $uname_m" ;;
esac

if [ "$(id -u)" -eq 0 ]; then ven_root='true'; else ven_root='false'; fi
if [ -t 0 ] && [ -t 1 ]; then ven_tty='true'; else ven_tty='false'; fi

printf '  os/arch: %s/%s\n' "$ven_os" "$ven_arch"
printf '  root:    %s\n' "$ven_root"
printf '  tty:     %s\n\n' "$ven_tty"

# ---------------------------------------------------------------------------
# Mode selection
# ---------------------------------------------------------------------------

if [ -z "$ven_mode" ]; then
    if [ "$ven_tty" = 'false' ]; then
        ven_mode='user'
        info '[1/6] No mode supplied + non-interactive shell => defaulting to "user".'
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

info "[2/6] Fetching release metadata: $api_url"
# shellcheck disable=SC2090
release_json="$(sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth '$api_url'")"

tag_name="$(printf '%s' "$release_json" | sed -n 's/.*"tag_name"[^"]*"\([^"]*\)".*/\1/p' | head -n1)"
[ -n "$tag_name" ] || err "could not parse tag_name from release JSON"
info "  resolved tag: $tag_name"

setup_asset_name="ven-setup-${ven_os}-${ven_arch}"
tar_asset_name="ven-${ven_os}-${ven_arch}.tar.gz"
sums_asset_name='SHA256SUMS'

# Pull asset download URLs by name without depending on jq (busybox / Alpine
# safe). GitHub's release API returns minified single-line JSON, so we cannot
# rely on line-oriented parsing of name + browser_download_url pairs. Instead
# we exploit the fact that every asset download URL ends with the asset's
# filename:
#   https://github.com/<repo>/releases/download/<tag>/<asset_name>
# Grep all such URLs out of the response and pick the one whose final path
# segment matches the requested asset name. Asset names in our naming
# contract are plain ASCII, so URL-encoding edge cases do not apply.
find_asset_url() {
    asset_name="$1"
    printf '%s' "$release_json" \
        | grep -oE 'https://github\.com/[^"[:space:]]+/releases/download/[^"[:space:]]+/[^"[:space:]]+' \
        | awk -v name="$asset_name" -F/ '$NF == name { print; exit }'
}

setup_url=''
if [ "$ven_force_replicate" != 'true' ]; then
    setup_url="$(find_asset_url "$setup_asset_name")"
fi
tar_url="$(find_asset_url "$tar_asset_name")"
sums_url="$(find_asset_url "$sums_asset_name")"

if [ -z "$setup_url" ] && [ -z "$tar_url" ]; then
    err "release $tag_name has neither '$setup_asset_name' nor '$tar_asset_name'"
fi

if [ -n "$setup_url" ]; then
    use_delegate='true'
    asset_name="$setup_asset_name"
    asset_url="$setup_url"
    info "  path:    Delegate ($asset_name)"
else
    use_delegate='false'
    asset_name="$tar_asset_name"
    asset_url="$tar_url"
    info "  path:    Replicate ($asset_name)"
fi

# ---------------------------------------------------------------------------
# Temp scratch + download (with trap cleanup)
# ---------------------------------------------------------------------------

ven_tmp="$(mktemp -d -t ven-install-XXXXXX)"
cleanup() {
    if [ "$ven_dry_run" != 'true' ] && [ -d "$ven_tmp" ]; then
        rm -rf "$ven_tmp"
    fi
}
trap cleanup EXIT INT TERM

download_path="$ven_tmp/$asset_name"

printf '\n[3/6] Downloading %s\n' "$asset_name"
if [ "$ven_dry_run" != 'true' ]; then
    sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth -o '$download_path' '$asset_url'"
    bytes="$(wc -c <"$download_path" | tr -d ' ')"
    info "  saved: $download_path ($bytes bytes)"
else
    info '  [dry-run] skipped download'
fi

# ---------------------------------------------------------------------------
# SHA256 verify
# ---------------------------------------------------------------------------

sha256_tool=''
if command -v sha256sum >/dev/null 2>&1; then sha256_tool='sha256sum'
elif command -v shasum  >/dev/null 2>&1; then sha256_tool='shasum -a 256'
fi

if [ "$ven_no_verify" = 'true' ]; then
    info "[4/6] Skipping SHA256 verification (--no-verify / VEN_NO_VERIFY)"
elif [ -z "$sums_url" ]; then
    info "[4/6] Skipping SHA256 verification (SHA256SUMS not present in release)"
elif [ -z "$sha256_tool" ]; then
    info "[4/6] Skipping SHA256 verification (no sha256sum / shasum found)"
elif [ "$ven_dry_run" = 'true' ]; then
    info '[4/6] [dry-run] skipped SHA256 verification'
else
    info "[4/6] Verifying SHA256 against SHA256SUMS"
    sums_path="$ven_tmp/SHA256SUMS"
    sh -c "curl -fsSL -H 'User-Agent: ven-install.sh' $curl_auth -o '$sums_path' '$sums_url'"
    expected="$(awk -v n="$asset_name" '$2 == n || $2 == "*"n {print $1; exit}' "$sums_path")"
    [ -n "$expected" ] || err "SHA256SUMS did not contain an entry for $asset_name"
    actual="$($sha256_tool "$download_path" | awk '{print $1}')"
    [ "$actual" = "$expected" ] || err "SHA256 mismatch for $asset_name: expected $expected, got $actual"
    info "  ok  ($expected)"
fi

# ---------------------------------------------------------------------------
# Install: Delegate path
# ---------------------------------------------------------------------------

do_delegate() {
    setup_exe="$download_path"
    printf '\n[5/6] Delegating to ven-setup (%s)\n' "$ven_mode"
    if [ "$ven_dry_run" = 'true' ]; then
        info "  [dry-run] would run: $setup_exe --mode $ven_mode --no-input"
        return 0
    fi
    chmod +x "$setup_exe"
    "$setup_exe" --mode "$ven_mode" --no-input
}

# ---------------------------------------------------------------------------
# Install: Replicate path (port of src/bin/setup/unix.rs)
# ---------------------------------------------------------------------------

VEN_RC_BLOCK_START='# >>> ven-setup PATH >>>'
VEN_RC_BLOCK_END='# <<< ven-setup PATH <<<'

append_block_if_missing() {
    # $1 rc path, $2 block
    rc="$1"; block="$2"
    if grep -F "$VEN_RC_BLOCK_START" "$rc" >/dev/null 2>&1; then
        return 0
    fi
    # Ensure trailing newline before appending.
    if [ -s "$rc" ] && [ "$(tail -c1 "$rc"; echo x)" != "$(printf '\nx')" ]; then
        printf '\n' >> "$rc"
    fi
    printf '%s\n' "$block" >> "$rc"
}

ensure_user_rc_path() {
    install_dir="$1"
    block="$(printf '%s\nexport PATH="%s:$PATH"\n%s' "$VEN_RC_BLOCK_START" "$install_dir" "$VEN_RC_BLOCK_END")"
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
    install_dir="$1"
    mkdir -p /etc/profile.d
    cat > /etc/profile.d/ven.sh <<EOF
#!/bin/sh
# Installed by ven-setup
case ":\$PATH:" in
  *":$install_dir:"*) ;;
  *) export PATH="$install_dir:\$PATH" ;;
esac
EOF
    chmod 0755 /etc/profile.d/ven.sh
}

do_replicate() {
    printf '\n[5/6] Replicate install (%s) -- shell port of ven-setup unix logic\n' "$ven_mode"

    if [ "$ven_mode" = 'system' ]; then
        install_dir='/usr/local/bin'
    else
        install_dir="$HOME/.ven/bin"
    fi

    info "  [a] Extract tarball -> $ven_tmp/extract"
    if [ "$ven_dry_run" != 'true' ]; then
        mkdir -p "$ven_tmp/extract"
        tar -xzf "$download_path" -C "$ven_tmp/extract"
    else
        info '      [dry-run] skipped'
    fi

    info "  [b] Install binaries -> $install_dir"
    if [ "$ven_dry_run" != 'true' ]; then
        mkdir -p "$install_dir"
        install -m 0755 "$ven_tmp/extract/ven"          "$install_dir/ven"
        install -m 0755 "$ven_tmp/extract/ven-launcher" "$install_dir/ven-launcher"
    else
        info '      [dry-run] skipped'
    fi

    if [ "$ven_mode" = 'system' ]; then
        info '  [c] Write /etc/profile.d/ven.sh (idempotent PATH guard)'
        if [ "$ven_dry_run" != 'true' ]; then
            ensure_etc_profile_d_path "$install_dir"
        else
            info '      [dry-run] skipped'
        fi
        info '  [d] Skipping per-user shell hooks (system install)'
        info '      [HINT] Each user should run: ven setup'
    else
        info '  [c] Append PATH block to rc files'
        if [ "$ven_dry_run" != 'true' ]; then
            ensure_user_rc_path "$install_dir"
        else
            info '      [dry-run] skipped'
        fi
        info '  [d] Install shell hooks (ven setup)'
        if [ "$ven_dry_run" != 'true' ]; then
            "$install_dir/ven" setup
        else
            info '      [dry-run] skipped'
        fi
    fi
}

# ---------------------------------------------------------------------------
# Dispatch + verify
# ---------------------------------------------------------------------------

if [ "$use_delegate" = 'true' ]; then
    do_delegate
else
    do_replicate
fi

printf '\n[6/6] Verifying ven --version in a new process\n'
if [ "$ven_dry_run" != 'true' ]; then
    if [ "$ven_mode" = 'system' ]; then
        install_dir='/usr/local/bin'
    else
        install_dir="$HOME/.ven/bin"
    fi
    if PATH="$install_dir:$PATH" sh -c 'ven --version'; then
        :
    else
        err "verification failed"
    fi
else
    info '  [dry-run] skipped verification'
fi

printf '\nDone. Open a new terminal (or `exec $SHELL -l`) and run: ven --version\n'
