// Single source of truth for every external URL, contact address, and social
// link the site references. The components below all import from here, so when
// you adopt your real domain / repo / email, change this file and nothing else.
//
//   - GITHUB_URL    repo root used by Header "GitHub", Footer columns, "View
//                   source" CTAs, and changelog deep links.
//   - INSTALL.*     one-liner install commands. Keep `pipe` separate from the
//                   URL so the Install page can render them as code blocks
//                   without ambiguity.
//   - RELEASES_URL  /releases page (linked from Changelog + Install).
//   - ISSUES_URL    new-issue page (Languages "Request" CTA).
//   - CONTACT_EMAIL set to `null` to hide the Contact link in the Footer.
//   - SOCIALS       optional row; any entry can be omitted.

export const GITHUB_URL = 'https://github.com/bhuwanb23/ven'
export const RELEASES_URL = `${GITHUB_URL}/releases`

// Built with Vite `base` (/ven/ on GitHub Pages, / in dev). Must not use a
// leading-slash absolute path or production fetch hits the domain root (404).
export const RELEASES_MANIFEST_URL = `${import.meta.env.BASE_URL}releases-manifest.json`
export const ISSUES_URL = `${GITHUB_URL}/issues`
export const DISCUSSIONS_URL = `${GITHUB_URL}/discussions`
export const LICENSE_URL = `${GITHUB_URL}/blob/main/LICENSE`
export const REQUEST_LANGUAGE_URL = `${ISSUES_URL}/new?labels=runtime-request`

// Until `get.ven.sh` is provisioned, the install one-liners hit
// `raw.githubusercontent.com` directly so they resolve on day one of the
// release. Swap to the short domain in this single place once it's live.
const INSTALL_PS1_URL = `https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.ps1`
const INSTALL_SH_URL = `https://raw.githubusercontent.com/bhuwanb23/ven/main/scripts/install.sh`

export const INSTALL = {
  windows: {
    label: 'Windows',
    cmd: `irm ${INSTALL_PS1_URL} | iex`,
    note:
      'PowerShell 5.1 or 7+. Or download ven-setup-0.2.3-windows-x64.exe above. Installs to %USERPROFILE%\\.ven\\bin. Then ven shell install (once); upgrade with ven update.',
  },
  macos: {
    label: 'macOS',
    cmd: `curl -fsSL ${INSTALL_SH_URL} | sh`,
    note:
      'Or download ven-setup-0.2.3-macos-* above. Installs to ~/.ven/bin and adds PATH. Then ven shell install (once); upgrade with ven update.',
  },
  linux: {
    label: 'Linux',
    cmd: `curl -fsSL ${INSTALL_SH_URL} | sh`,
    note:
      'Or download ven-setup-0.2.3-linux-* above (chmod +x, then run). Bash / Zsh / Fish. Then ven shell install (once); upgrade with ven update.',
  },
  source: {
    label: 'From source',
    cmd: `git clone ${GITHUB_URL} && cd ven && cargo build --release`,
    note: 'Requires the Rust toolchain (rustup.rs). Two-pass build embeds ven + launcher into ven-setup.',
  },
}

export const PLATFORM_ORDER = ['windows', 'macos', 'linux', 'source']

// First-time hook, then self-update — surfaced on the landing demo section.
// Order matches the recommended workflow (setup once; `ven update` from v0.1.7+).
export const AFTER_INSTALL_COMMANDS = ['ven setup', 'ven update']

// Uninstall snippets shown on /install.
//
// SHAPE (since v0.1.7):
//
//   UNINSTALL.simple
//     The recommended path. A single `ven uninstall` invocation: shows a
//     dry-run plan, prompts for confirmation, then removes binary + every
//     runtime + cache + state + persisted VEN_HOME + pointer file + PATH
//     entries + rc-file blocks. Honors a relocated $VEN_HOME (`ven path
//     set D:\ven` → uninstall removes both `~/.ven` and `D:\ven`).
//
//   UNINSTALL.advanced
//     The original copy-paste snippets, kept for two recovery cases:
//       1. The `ven` binary is broken / missing from PATH.
//       2. You want to read the shell version before trusting the
//          Rust impl.
//     SYNC: scripts/uninstall.ps1 (windows), scripts/uninstall.sh
//     (macos/linux). The bundled `ven-uninstall.{ps1,sh}` in each
//     install dir is the same content as these blocks. Audit all four
//     when one changes.
//
// Each script handles BOTH install modes the installers support:
//
//                       USER (no admin)              SYSTEM (admin)
//   Windows binaries    %USERPROFILE%\.ven\bin       %ProgramFiles%\ven\bin
//   Windows PATH        HKCU\Environment\Path        HKLM\...\Path
//   Unix binaries       ~/.ven/bin                   /usr/local/bin/ven*
//   Unix PATH           ~/.bashrc / ~/.zshrc / ~/.profile (per-user PATH block)
//                       /etc/profile.d/ven.sh (system-wide PATH file)
//
// Three different `advanced` commands and not one universal one-liner because:
//
//   - Windows has no `rm -rf` or `sed`. PowerShell-native verbs (Remove-Item +
//     [Environment]::SetEnvironmentVariable) are the only thing that works on
//     a stock install of PowerShell 5.1 / 7+, which is what the install
//     one-liner targets.
//   - macOS ships BSD sed, which **requires** an explicit backup-extension
//     argument (`-i ''`). The Linux GNU sed form (`-i` alone) silently
//     truncates the file to nothing on macOS, so reusing the Linux command
//     there would *break* the user's rc files instead of cleaning them up.
//   - Linux distros all ship GNU sed; `-i` without an argument is correct.

const UNINSTALL_ADVANCED = {
  windows: {
    label: 'Windows · PowerShell',
    prompt: 'PS>',
    note:
      'Cleans BOTH install modes idempotently. Run once unprivileged for the user-mode portion; if a system install is detected, re-run in an elevated PowerShell to finish. Reports exactly what was removed so you can see the script worked. Open a new terminal afterwards so the cleaned PATH takes effect.',
    cmd: [
      `# Helper: normalise a PATH entry for comparison (expand env vars, trim`,
      `# whitespace + trailing slashes, lowercase). Critical for catching`,
      `# %ProgramFiles%-style unexpanded entries, trailing-space / trailing-`,
      `# backslash variants, and case-mismatched paths.`,
      `function _Norm([string]$s) {`,
      `  if ([string]::IsNullOrWhiteSpace($s)) { return '' }`,
      `  return ([Environment]::ExpandEnvironmentVariables($s.Trim())).TrimEnd('\\','/').ToLowerInvariant()`,
      `}`,
      `# Helper: remove every PATH entry matching one of \`$targets from the given`,
      `# scope. Returns the array of removed entries (empty if no change). Always`,
      `# runs to completion regardless of directory state.`,
      `function _StripPath([string]$scope, [string[]]$targets) {`,
      `  $current = [Environment]::GetEnvironmentVariable('Path', $scope)`,
      `  if (-not $current) { return @() }`,
      `  $targetSet = @{}; foreach ($t in $targets) { $targetSet[(_Norm $t)] = $true }`,
      `  $kept = @(); $removed = @()`,
      `  foreach ($e in ($current -split ';')) {`,
      `    if ([string]::IsNullOrWhiteSpace($e)) { continue }`,
      `    if ($targetSet.ContainsKey((_Norm $e))) { $removed += $e } else { $kept += $e }`,
      `  }`,
      `  if ($removed.Count -gt 0) {`,
      `    try { [Environment]::SetEnvironmentVariable('Path', ($kept -join ';'), $scope) }`,
      `    catch { Write-Warning ("Could not update $scope PATH: " + $_.Exception.Message); return @() }`,
      `  }`,
      `  return $removed`,
      `}`,
      ``,
      `# 1. User install (no admin needed)`,
      `$userRoot = Join-Path $env:USERPROFILE '.ven'`,
      `$userBin  = Join-Path $userRoot 'bin'`,
      `if (Test-Path $userRoot) {`,
      `  Remove-Item -Recurse -Force $userRoot -ErrorAction SilentlyContinue`,
      `  if (-not (Test-Path $userRoot)) { Write-Host "Removed user install: $userRoot" }`,
      `}`,
      `$ur = _StripPath 'User' @($userBin)`,
      `if ($ur) { Write-Host ("Stripped from User PATH: " + ($ur -join '; ')) }`,
      ``,
      `# 2. System install at %ProgramFiles%\\ven (admin only) — directory and`,
      `# PATH entry checked INDEPENDENTLY so a half-finished previous run still`,
      `# converges to clean state.`,
      `$sysRoot = Join-Path $env:ProgramFiles 'ven'`,
      `$sysBin  = Join-Path $sysRoot 'bin'`,
      `$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)`,
      `$sysDirExists = Test-Path $sysRoot`,
      `$mp = [Environment]::GetEnvironmentVariable('Path', 'Machine')`,
      `$sysPathHasIt = $false; $sysBinNorm = _Norm $sysBin`,
      `if ($mp) { foreach ($e in ($mp -split ';')) { if ((_Norm $e) -eq $sysBinNorm) { $sysPathHasIt = $true; break } } }`,
      ``,
      `if ($sysDirExists -or $sysPathHasIt) {`,
      `  if ($isAdmin) {`,
      `    if ($sysDirExists) {`,
      `      Remove-Item -Recurse -Force $sysRoot -ErrorAction SilentlyContinue`,
      `      if (-not (Test-Path $sysRoot)) { Write-Host "Removed system install: $sysRoot" }`,
      `      else { Write-Warning "Could not remove $sysRoot (file in use? close all ven shells)" }`,
      `    }`,
      `    $sr = _StripPath 'Machine' @($sysBin)`,
      `    if ($sr) { Write-Host ("Stripped from Machine PATH: " + ($sr -join '; ')) }`,
      `  } else {`,
      `    Write-Warning "System install detected. Re-run in an elevated PowerShell to remove it cleanly:"`,
      `    if ($sysDirExists)  { Write-Warning "  - Directory still present: $sysRoot" }`,
      `    if ($sysPathHasIt)  { Write-Warning "  - Machine PATH still contains: $sysBin" }`,
      `  }`,
      `}`,
      ``,
      `Write-Host ""; Write-Host "Done. Open a NEW terminal so the cleaned PATH takes effect."`,
    ].join('\n'),
  },
  macos: {
    label: 'macOS · bash / zsh',
    prompt: '$',
    note:
      'Cleans BOTH install modes idempotently. macOS ships BSD sed, which needs the empty backup-extension (-i \'\'). The user portion always runs; the system portion auto-uses sudo, fires whenever EITHER the binary OR the /etc/profile.d entry is found, and reports what it removed.',
    cmd: [
      `# 1. User install (no sudo needed) — directory + rc-file PATH block`,
      `[ -d ~/.ven ] && rm -rf ~/.ven && echo 'Removed: ~/.ven'`,
      `for rc in ~/.bashrc ~/.zshrc ~/.zprofile ~/.bash_profile ~/.profile; do`,
      `  [ -f "$rc" ] && sed -i '' '/\\.ven\\/bin/d' "$rc" 2>/dev/null`,
      `done`,
      ``,
      `# 2. System install (sudo) — runs if EITHER artefact is present so a`,
      `#    half-finished previous run still converges. Each rm -f is a no-op`,
      `#    when its target is already gone.`,
      `if [ -e /usr/local/bin/ven ] || [ -e /usr/local/bin/ven-launcher ] || [ -e /usr/local/bin/ven-setup ] || [ -e /etc/profile.d/ven.sh ]; then`,
      `  sudo rm -fv /usr/local/bin/ven /usr/local/bin/ven-launcher /usr/local/bin/ven-setup /etc/profile.d/ven.sh`,
      `fi`,
      `hash -r 2>/dev/null`,
      `echo 'Done. Open a NEW terminal so the cleaned PATH takes effect.'`,
    ].join('\n'),
  },
  linux: {
    label: 'Linux · bash / zsh',
    prompt: '$',
    note:
      'Cleans BOTH install modes idempotently. User portion runs unprivileged; system portion uses sudo, fires whenever EITHER the binary OR the /etc/profile.d entry is found, and reports what it removed.',
    cmd: [
      `# 1. User install (no sudo needed) — directory + rc-file PATH block`,
      `[ -d ~/.ven ] && rm -rf ~/.ven && echo 'Removed: ~/.ven'`,
      `for rc in ~/.bashrc ~/.zshrc ~/.bash_profile ~/.profile; do`,
      `  [ -f "$rc" ] && sed -i '/\\.ven\\/bin/d' "$rc" 2>/dev/null`,
      `done`,
      ``,
      `# 2. System install (sudo) — runs if EITHER artefact is present so a`,
      `#    half-finished previous run still converges. Each rm -f is a no-op`,
      `#    when its target is already gone.`,
      `if [ -e /usr/local/bin/ven ] || [ -e /usr/local/bin/ven-launcher ] || [ -e /usr/local/bin/ven-setup ] || [ -e /etc/profile.d/ven.sh ]; then`,
      `  sudo rm -fv /usr/local/bin/ven /usr/local/bin/ven-launcher /usr/local/bin/ven-setup /etc/profile.d/ven.sh`,
      `fi`,
      `hash -r 2>/dev/null`,
      `echo 'Done. Open a NEW terminal so the cleaned PATH takes effect.'`,
    ].join('\n'),
  },
}

// Recommended path (since v0.1.7): a single first-class CLI command.
// `ven uninstall` is dry-run-capable, JSON-emitting, scope-flagged
// (--user-only / --system-only), and idempotent — re-running a partial
// uninstall converges to a clean state. See docs/cmds/uninstall.md.
const UNINSTALL_SIMPLE = {
  label: 'ven 0.2.3+',
  prompt: '$',
  note:
    'Native command. Prints a dry-run plan, prompts before nuking, removes everything (binary + every runtime + cache + state + persisted VEN_HOME + pointer file + PATH entries). Use `--dry-run` first to see exactly what would be removed; `-y` skips the prompt for CI.',
  cmd: [
    `# Preview the plan first (recommended)`,
    `ven uninstall --dry-run`,
    ``,
    `# Then do it for real (asks "Permanently remove ven and all installed runtimes? [y/N]")`,
    `ven uninstall`,
    ``,
    `# Non-interactive / CI`,
    `ven uninstall -y                # skip the confirm prompt`,
    `ven uninstall --user-only       # skip the system layer (no sudo needed)`,
    `ven uninstall --json --dry-run  # capture the plan as JSON for a CI gate`,
  ].join('\n'),
}

export const UNINSTALL = {
  simple: UNINSTALL_SIMPLE,
  advanced: UNINSTALL_ADVANCED,
}

export const UNINSTALL_ORDER = ['windows', 'macos', 'linux']

// `null` hides the Contact link in the Footer until a real address exists.
export const CONTACT_EMAIL = null

export const SOCIALS = [
  { id: 'github', label: 'GitHub', href: GITHUB_URL },
  { id: 'issues', label: 'Issues', href: ISSUES_URL },
  { id: 'discussions', label: 'Discussions', href: DISCUSSIONS_URL },
]

/**
 * Best-effort browser platform detection for the hero install tab.
 * Falls back to `windows` (the most common dev OS in the userbase) when the
 * userAgent string yields nothing reliable, e.g. during SSR or in headless
 * preview environments.
 */
export function detectPlatform() {
  if (typeof navigator === 'undefined') return 'windows'
  const uaData = navigator.userAgentData
  if (uaData?.platform) {
    const p = uaData.platform.toLowerCase()
    if (p.includes('win')) return 'windows'
    if (p.includes('mac')) return 'macos'
    if (p.includes('linux')) return 'linux'
  }
  const ua = (navigator.userAgent || '').toLowerCase()
  if (ua.includes('windows')) return 'windows'
  if (ua.includes('mac os') || ua.includes('macintosh')) return 'macos'
  if (ua.includes('linux') || ua.includes('x11')) return 'linux'
  return 'windows'
}
