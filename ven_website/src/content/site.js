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
    note: 'PowerShell 5.1 or 7+. Installs to %USERPROFILE%\\.ven\\bin.',
  },
  macos: {
    label: 'macOS',
    cmd: `curl -fsSL ${INSTALL_SH_URL} | sh`,
    note: 'Installs to ~/.ven/bin/ven and adds it to PATH automatically.',
  },
  linux: {
    label: 'Linux',
    cmd: `curl -fsSL ${INSTALL_SH_URL} | sh`,
    note: 'Bash / Zsh / Fish supported. Installs to ~/.ven/bin/ven.',
  },
  source: {
    label: 'From source',
    cmd: `git clone ${GITHUB_URL} && cd ven && cargo build --release`,
    note: 'Requires the Rust toolchain (rustup.rs). Two-pass build embeds ven + launcher into ven-setup.',
  },
}

export const PLATFORM_ORDER = ['windows', 'macos', 'linux', 'source']

// Uninstall snippets shown on /install. Three real, working scripts — one per
// OS family — that actually remove ven from a fresh machine. Kept here (not
// inline in Install.jsx) so a future cleanup pass touches a single file.
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
// The first half of every script is unprivileged (cleans the user-mode
// install). The second half detects a system install and either cleans it
// (if running elevated / via sudo) or prints a clear "re-run elevated"
// message. So copy-paste once unprivileged, optionally a second time
// elevated, and ven is fully gone — no detective work required.
//
// Why three different commands and not one universal one-liner:
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
//
// Each `cmd` is multi-line so it survives copy/paste cleanly into the target
// shell. The trailing `2>/dev/null` (Unix) and `-ErrorAction SilentlyContinue`
// (Windows) swallow "not found" noise when only one of the two install modes
// was actually used.
export const UNINSTALL = {
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
