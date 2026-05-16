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
      'Cleans both modes. The user portion always runs; the system portion only triggers if %ProgramFiles%\\ven exists, and asks for elevation when needed. Open a new terminal so the cleaned PATH takes effect.',
    cmd: [
      `# 1. User install (no admin needed)`,
      `$userRoot = Join-Path $env:USERPROFILE '.ven'`,
      `Remove-Item -Recurse -Force $userRoot -ErrorAction SilentlyContinue`,
      `$userBin = Join-Path $userRoot 'bin'`,
      `$p = [Environment]::GetEnvironmentVariable('Path', 'User')`,
      `if ($p) { [Environment]::SetEnvironmentVariable('Path', (($p -split ';') | ? { $_ -and $_.TrimEnd('\\') -ine $userBin }) -join ';', 'User') }`,
      ``,
      `# 2. System install at %ProgramFiles%\\ven (admin only)`,
      `$sysRoot = Join-Path $env:ProgramFiles 'ven'`,
      `if (Test-Path $sysRoot) {`,
      `  $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)`,
      `  if ($isAdmin) {`,
      `    Remove-Item -Recurse -Force $sysRoot`,
      `    $sysBin = Join-Path $sysRoot 'bin'`,
      `    $mp = [Environment]::GetEnvironmentVariable('Path', 'Machine')`,
      `    if ($mp) { [Environment]::SetEnvironmentVariable('Path', (($mp -split ';') | ? { $_ -and $_.TrimEnd('\\') -ine $sysBin }) -join ';', 'Machine') }`,
      `    Write-Host "Removed system install: $sysRoot"`,
      `  } else {`,
      `    Write-Warning "System install at $sysRoot needs admin. Re-run this script in an elevated PowerShell to remove it."`,
      `  }`,
      `}`,
    ].join('\n'),
  },
  macos: {
    label: 'macOS · bash / zsh',
    prompt: '$',
    note:
      'macOS ships BSD sed, which needs the empty backup-extension (-i \'\'). The user portion always runs; the system portion auto-uses sudo and only fires if /usr/local/bin/ven actually exists.',
    cmd: [
      `# 1. User install (no sudo needed)`,
      `rm -rf ~/.ven`,
      `sed -i '' '/\\.ven\\/bin/d' ~/.bashrc ~/.zshrc ~/.zprofile ~/.profile 2>/dev/null`,
      ``,
      `# 2. System install (sudo only if it exists)`,
      `if [ -e /usr/local/bin/ven ] || [ -e /etc/profile.d/ven.sh ]; then`,
      `  sudo rm -f /usr/local/bin/ven /usr/local/bin/ven-launcher /usr/local/bin/ven-setup /etc/profile.d/ven.sh`,
      `fi`,
      `hash -r 2>/dev/null`,
    ].join('\n'),
  },
  linux: {
    label: 'Linux · bash / zsh',
    prompt: '$',
    note:
      'Works on every distro the release matrix tests (Debian, Ubuntu, Fedora, Arch, Alpine). User portion runs unprivileged; system portion uses sudo and only fires if /usr/local/bin/ven actually exists.',
    cmd: [
      `# 1. User install (no sudo needed)`,
      `rm -rf ~/.ven`,
      `sed -i '/\\.ven\\/bin/d' ~/.bashrc ~/.zshrc ~/.profile 2>/dev/null`,
      ``,
      `# 2. System install (sudo only if it exists)`,
      `if [ -e /usr/local/bin/ven ] || [ -e /etc/profile.d/ven.sh ]; then`,
      `  sudo rm -f /usr/local/bin/ven /usr/local/bin/ven-launcher /usr/local/bin/ven-setup /etc/profile.d/ven.sh`,
      `fi`,
      `hash -r 2>/dev/null`,
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
