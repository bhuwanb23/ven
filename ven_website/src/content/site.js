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

export const GITHUB_URL = 'https://github.com/yourorg/ven'
export const RELEASES_URL = `${GITHUB_URL}/releases`
export const ISSUES_URL = `${GITHUB_URL}/issues`
export const DISCUSSIONS_URL = `${GITHUB_URL}/discussions`
export const LICENSE_URL = `${GITHUB_URL}/blob/main/LICENSE`
export const REQUEST_LANGUAGE_URL = `${ISSUES_URL}/new?labels=runtime-request`

export const INSTALL = {
  windows: {
    label: 'Windows',
    cmd: 'irm https://get.ven.sh/install.ps1 | iex',
    note: 'PowerShell 5.1 or 7+. Installs to %USERPROFILE%\\.ven\\bin.',
  },
  macos: {
    label: 'macOS',
    cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh',
    note: 'Installs to ~/.ven/bin/ven and adds it to PATH automatically.',
  },
  linux: {
    label: 'Linux',
    cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh',
    note: 'Bash / Zsh / Fish supported. Installs to ~/.ven/bin/ven.',
  },
  source: {
    label: 'From source',
    cmd: `git clone ${GITHUB_URL} && cd ven && cargo build --release`,
    note: 'Requires the Rust toolchain (rustup.rs). Two-pass build embeds ven + launcher into ven-setup.',
  },
}

export const PLATFORM_ORDER = ['windows', 'macos', 'linux', 'source']

export const CONTACT_EMAIL = 'hello@ven.sh'

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
