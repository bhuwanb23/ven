import { useEffect, useState } from 'react'
import clsx from 'clsx'
import Icon from '../components/ui/Icon.jsx'
import CodeBlock from '../components/ui/CodeBlock.jsx'
import GlassCard from '../components/ui/GlassCard.jsx'
import Terminal from '../components/ui/Terminal.jsx'
import Reveal from '../components/effects/Reveal.jsx'
import {
  INSTALL,
  PLATFORM_ORDER,
  UNINSTALL,
  UNINSTALL_ORDER,
  GITHUB_URL,
  RELEASES_URL,
  RELEASES_MANIFEST_URL,
  detectPlatform,
} from '../content/site.js'

const FAQ = [
  {
    q: 'How fast is the auto-switching hook?',
    a: 'Sub-50ms on a warm cache. The shell hook is a single Rust binary call that reads `ven.toml`, resolves the runtime against `$VEN_HOME` (default `~/.ven`), and exports the new PATH — no shim, no fork-per-command tax.',
  },
  {
    q: 'Does it support Windows?',
    a: 'Yes — first-class. PowerShell 5.1 + 7+ hook, a portable `ven-launcher.exe` for locked-down corporate machines (shipped as a discoverable `ven-launcher-windows-{arch}.zip` bundle that includes a double-clickable `Start ven.cmd` shim since v0.1.2), and a UAC-aware `ven-setup.exe` for system installs.',
  },
  {
    q: 'Can it coexist with nvm / pyenv?',
    a: 'Yes, but auto-switching hooks will fight for PATH precedence. Recommended migration: install ven, run `ven status` to confirm your projects resolve, then remove the older tool to avoid double resolution.',
  },
  {
    q: 'Is there a GUI?',
    a: 'The day-to-day `ven` CLI stays terminal-first. Since v0.2.0, `ven-setup` ships a native GUI wizard (install mode, storage path, PATH/hook toggles, optional runtime pre-install). Use `ven-setup --cli` for SSH, CI, or headless servers.',
  },
  {
    q: 'How is it different from mise / asdf?',
    a: 'mise and asdf manage runtime versions only. ven adds a pre-install dependency graph engine, OSV-backed CVE scanning, endoflife.date EOL alerts, a unified `ven add` package surface across 8 ecosystems, deterministic ven.lock with SHA-256 content hash, and a portable launcher for restricted environments.',
  },
  {
    q: 'Where does ven store data?',
    a: 'Resolved on every run via `VEN_HOME` (4-tier precedence: `$VEN_HOME` → `$VEN_STORAGE_PATH` → `<launcher-dir>/.ven` → `~/.ven`). Binaries live in `<root>/bin`, runtimes in `<root>/<lang>/<version>/`, and a SQLite cache at `<root>/cache/` for OSV / EOL / docs lookups. Drop a `.ven/` folder next to `ven-launcher` for fully portable USB-stick installs — no `~/.ven` writes, no PATH edits.',
  },
  {
    q: '"error sending request for url" behind Zscaler / corporate proxy?',
    a: 'Upgrade to **v0.1.3 or newer**. Enterprise proxies (Zscaler, Netskope, Bluecoat) MITM HTTPS using a private root CA installed in the OS trust store. ven ≤ v0.1.2 used only the bundled Mozilla roots and ignored the OS store, so `ven install python` failed even though the same URL opened in your browser. v0.1.3 enables `rustls-tls-native-roots` and merges both root pools — no flags, no env vars, no custom CA file to maintain. The same binary works at home and behind Zscaler.',
  },
  {
    q: 'What happens if I re-run the installer over an existing version?',
    a: 'Since v0.1.5 the installer detects every existing ven on disk (`%USERPROFILE%\\.ven\\bin` and `%ProgramFiles%\\ven\\bin` on Windows; `~/.ven/bin` and `/usr/local/bin/ven` on Unix) and prints what it finds. Same mode + same version → exits cleanly with "nothing to do". Same mode + a different version → prompts to upgrade. Different mode (e.g. user install requested while a system install already exists) → warns that PATH precedence will shadow one of the two binaries, then asks you to confirm. In CI / piped contexts there is no TTY to prompt, so the installer aborts safely; set `VEN_FORCE_INSTALL=true` (or pass `-Force` / `--force`) to skip the prompt.',
  },
  {
    q: 'How do I upgrade ven once it\'s installed?',
    a: 'Run `ven update`. Since v0.1.7 ven self-updates: it detects whether you installed in user-mode (`~/.ven/bin`) or system-mode (`%ProgramFiles%\\ven\\bin` / `/usr/local/bin`), downloads the platform-specific combined release asset, verifies it against the release\'s SHA256SUMS manifest, and swaps both `ven` and `ven-launcher` in place. System installs auto-elevate through UAC on Windows or `sudo` on Unix. Use `ven update --check` to see what\'s available without applying, or `ven update --version v0.1.6` to roll back. Do NOT confuse it with `ven upgrade` — that one upgrades project packages (npm / pip / cargo / …), not ven itself.',
  },
]

// Display labels + tone for the per-kind groups of the downloads table.
// Using a fixed map keeps the order stable: combined first (the default
// install path), then the discoverable portable bundle, then the standalone
// installer.
const KIND_META = {
  combined: {
    label: 'Combined archive',
    tagline: 'Used by the install one-liners. Contains ven + ven-launcher.',
    accent: 'text-primary-fixed-dim',
  },
  launcher: {
    label: 'Portable launcher bundle',
    tagline:
      'Corporate / Zscaler friendly. Includes a double-clickable terminal shim (Start ven.cmd / Start ven.command / start-ven.sh) so non-CLI users can extract and run with one click. No admin, no PATH edits.',
    accent: 'text-secondary-fixed-dim',
  },
  setup: {
    label: 'Standalone installer',
    tagline:
      'Self-contained ven-setup opens an eight-screen GUI wizard (v0.2+): storage path, PATH/hook toggles, optional runtime pre-install, live progress. UAC on Windows, sudo on Unix. Use --cli for headless/SSH.',
    accent: 'text-tertiary-fixed-dim',
  },
}

const KIND_ORDER = ['setup', 'combined', 'launcher']

// ---------------------------------------------------------------------------
// Corporate / portable download card metadata.
//
// We map each `os_label` to the user-facing OS name, the default arch we
// recommend (x64 for Windows/Linux, arm64 for macOS since Apple Silicon is
// the modern default), and the per-OS terminal-shim filename so the UI can
// say "Double-click 'Start ven.cmd'" instead of forcing the visitor to
// guess what to click after extracting.
// ---------------------------------------------------------------------------
const OS_LABEL = { windows: 'Windows', macos: 'macOS', linux: 'Linux' }
const OS_DEFAULT_ARCH = { windows: 'x64', macos: 'arm64', linux: 'x64' }
const OS_SHIM = {
  windows: 'Start ven.cmd',
  macos: 'Start ven.command',
  linux: 'start-ven.sh',
}
const OS_ICON = { windows: 'window', macos: 'laptop_mac', linux: 'terminal' }
const OS_ORDER = ['windows', 'macos', 'linux']
const ARCH_ORDER = ['x64', 'arm64']

function findLauncherAsset(data, os, arch) {
  if (!data) return null
  const file = `ven-launcher-${os}-${arch}.${os === 'windows' ? 'zip' : 'tar.gz'}`
  return data.assets.find((a) => a.kind === 'launcher' && a.file === file) ?? null
}

function findSetupAsset(data, os, arch) {
  if (!data) return null
  const file =
    os === 'windows' ? `ven-setup-${os}-${arch}.exe` : `ven-setup-${os}-${arch}`
  return data.assets.find((a) => a.kind === 'setup' && a.file === file) ?? null
}

/** Friendly label shown on the primary download button (Python-style naming). */
function setupLabel(asset, data) {
  if (asset?.displayName) return asset.displayName
  if (!data) return 'ven-setup'
  const v = String(data.version).replace(/^v/, '')
  const os = asset?.file?.includes('windows')
    ? 'windows'
    : asset?.file?.includes('macos')
      ? 'macos'
      : 'linux'
  const arch = asset?.file?.includes('arm64') ? 'arm64' : 'x64'
  return os === 'windows'
    ? `ven-setup-${v}-${os}-${arch}.exe`
    : `ven-setup-${v}-${os}-${arch}`
}

// Lifted out of `DownloadsTable` so multiple sections (Corporate one-click
// download CTA + the full Direct-downloads table) can share a single fetch
// instead of racing the same request twice.
function useReleasesManifest() {
  const [data, setData] = useState(null)
  const [err, setErr] = useState(null)
  useEffect(() => {
    let cancelled = false
    fetch(RELEASES_MANIFEST_URL)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`)
        return r.json()
      })
      .then((j) => !cancelled && setData(j))
      .catch((e) => !cancelled && setErr(e.message))
    return () => {
      cancelled = true
    }
  }, [])
  return { data, err }
}

function PlatformTabs({ active, onChange }) {
  return (
    <div className="flex flex-wrap justify-center gap-2 mb-12">
      {PLATFORM_ORDER.map((id) => {
        const p = INSTALL[id]
        const isActive = id === active
        return (
          <button
            key={id}
            type="button"
            onClick={() => onChange(id)}
            className={clsx(
              'px-6 py-2 glass-surface transition-all border-b-2',
              isActive
                ? 'border-primary-fixed-dim text-primary-fixed-dim font-bold'
                : 'border-transparent text-on-surface-variant hover:text-primary-fixed-dim'
            )}
          >
            {p.label}
          </button>
        )
      })}
    </div>
  )
}

function DownloadsTable({ data, err }) {
  if (err) {
    return (
      <p className="text-on-surface-variant text-sm opacity-70">
        Releases manifest unavailable — see{' '}
        <a className="text-primary-fixed-dim hover:underline" href={RELEASES_URL}>
          GitHub Releases
        </a>{' '}
        for the latest assets.
      </p>
    )
  }

  if (!data) {
    return <p className="text-on-surface-variant text-sm opacity-70">Loading release assets…</p>
  }

  // Group assets by `kind`. Older manifests without the `kind` field fall
  // back to the `combined` bucket so the page still renders for stale data.
  const grouped = data.assets.reduce((acc, d) => {
    const k = d.kind ?? 'combined'
    ;(acc[k] ??= []).push(d)
    return acc
  }, {})

  return (
    <>
      <h2 className="font-headline-md text-headline-md mb-8 flex items-center gap-3 flex-wrap">
        Direct downloads{' '}
        <span className="font-mono text-sm text-primary-fixed-dim">v{data.version}</span>
        <span className="font-mono text-xs text-on-surface-variant opacity-60">· {data.date}</span>
        <a
          href={data.notesUrl}
          target="_blank"
          rel="noreferrer"
          className="ml-auto text-sm text-on-surface-variant hover:text-primary-fixed-dim transition-colors"
        >
          release notes →
        </a>
      </h2>

      <div className="space-y-12">
        {KIND_ORDER.filter((k) => (grouped[k]?.length ?? 0) > 0).map((k) => {
          const meta = KIND_META[k]
          return (
            <section key={k}>
              <header className="mb-3 flex items-baseline gap-3 flex-wrap">
                <h3 className={clsx('font-headline-md text-lg font-bold', meta.accent)}>
                  {meta.label}
                </h3>
                <span className="text-xs text-on-surface-variant opacity-70">
                  {meta.tagline}
                </span>
              </header>
              <div className="overflow-x-auto">
                <table className="w-full text-left border-collapse font-mono text-sm">
                  <thead>
                    <tr className="border-b border-outline-variant/30 text-on-surface-variant uppercase text-[10px] tracking-widest">
                      <th className="py-3 px-2">Platform</th>
                      <th className="py-3 px-2">Artifact</th>
                      <th className="py-3 px-2">SHA-256</th>
                      <th className="py-3 px-2 text-right">Size</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-outline-variant/10">
                    {grouped[k].map((d) => (
                      <tr key={d.file} className="hover:bg-surface-container-low transition-colors">
                        <td className="py-3 px-2 font-bold text-primary-fixed-dim">{d.platform}</td>
                        <td className="py-3 px-2">
                          <a
                            href={d.url}
                            target="_blank"
                            rel="noreferrer"
                            title={d.displayName && d.displayName !== d.file ? d.file : undefined}
                            className="hover:text-primary-fixed-dim underline-offset-4 hover:underline"
                          >
                            {d.displayName ?? d.file}
                          </a>
                        </td>
                        <td className="py-3 px-2 opacity-60 break-all">{d.sha256}</td>
                        <td className="py-3 px-2 text-right">{d.size}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          )
        })}
      </div>

      <p className="mt-6 text-xs text-on-surface-variant opacity-70">
        Every asset ships with a `.sha256` sidecar and an aggregate `SHA256SUMS` manifest. The install
        scripts verify hashes automatically before extraction.
      </p>
    </>
  )
}

// Primary installer download — Python / Node style OS picker + big button.
function InstallerDownload({ data, err }) {
  const [os, setOs] = useState(() => {
    const detected = detectPlatform()
    return OS_LABEL[detected] ? detected : 'windows'
  })
  const [arch, setArch] = useState(() => OS_DEFAULT_ARCH[os] ?? 'x64')

  function pickOs(next) {
    setOs(next)
    setArch(OS_DEFAULT_ARCH[next] ?? 'x64')
  }

  const asset = findSetupAsset(data, os, arch)
  const archesAvailable = ARCH_ORDER.filter((a) => findSetupAsset(data, os, a))
  const label = setupLabel(asset, data)

  return (
    <div className="glass-surface p-8 border-l-4 border-primary-fixed-dim rounded-r-xl mb-8">
      <div className="flex flex-col md:flex-row gap-8 items-start">
        <div className="grow w-full">
          <div className="flex items-center gap-2 mb-2 flex-wrap">
            <span className="text-[10px] uppercase tracking-widest font-bold text-primary-fixed-dim bg-primary-fixed-dim/10 px-2 py-0.5 rounded">
              {data ? `v${data.version} · GUI wizard` : 'GUI wizard'}
            </span>
            <span className="text-[10px] uppercase tracking-widest text-on-surface-variant opacity-70">
              recommended for most users
            </span>
          </div>
          <h2 className="font-headline-md text-headline-md mb-2 text-primary-fixed-dim">
            Download Ven Setup
          </h2>
          <p className="text-on-surface-variant text-body-base mb-6">
            Double-click the installer for your platform. The wizard walks you through install
            mode, where to store runtimes (<code className="text-on-surface">$VEN_HOME</code>),
            PATH + shell hook options, and optional language pre-installs — no terminal required.
          </p>

          <div className="flex flex-wrap gap-2 mb-3">
            {OS_ORDER.map((id) => (
              <button
                key={id}
                type="button"
                onClick={() => pickOs(id)}
                className={clsx(
                  'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-mono transition-colors border',
                  os === id
                    ? 'border-primary-fixed-dim text-primary-fixed-dim bg-primary-fixed-dim/10'
                    : 'border-outline-variant/40 text-on-surface-variant hover:text-on-surface'
                )}
              >
                <Icon name={OS_ICON[id]} className="text-base" />
                {OS_LABEL[id]}
              </button>
            ))}
          </div>

          {archesAvailable.length > 1 && (
            <div className="flex items-center gap-2 mb-5 text-xs">
              <span className="text-on-surface-variant opacity-70 uppercase tracking-widest">
                arch
              </span>
              {archesAvailable.map((a) => (
                <button
                  key={a}
                  type="button"
                  onClick={() => setArch(a)}
                  className={clsx(
                    'px-2.5 py-0.5 rounded-full font-mono uppercase tracking-widest border transition-colors',
                    arch === a
                      ? 'border-primary-fixed-dim text-primary-fixed-dim'
                      : 'border-outline-variant/30 text-on-surface-variant hover:text-on-surface'
                  )}
                >
                  {a}
                </button>
              ))}
            </div>
          )}

          <div className="mb-6">
            {err ? (
              <p className="text-sm text-on-surface-variant opacity-70">
                Releases manifest unavailable — see{' '}
                <a className="text-primary-fixed-dim hover:underline" href={RELEASES_URL}>
                  GitHub Releases
                </a>
                .
              </p>
            ) : !data ? (
              <p className="text-sm text-on-surface-variant opacity-70">Loading installer…</p>
            ) : !asset ? (
              <p className="text-sm text-on-surface-variant opacity-70">
                No installer for {OS_LABEL[os]} {arch} in v{data.version}.
              </p>
            ) : (
              <a
                href={asset.url}
                className="group inline-flex items-center gap-3 px-6 py-4 rounded-xl bg-primary-fixed-dim text-on-primary-fixed font-bold shadow-lg hover:shadow-xl hover:-translate-y-0.5 transition-all"
              >
                <Icon name="download" className="text-2xl" />
                <span className="text-left">
                  <span className="block text-base font-mono">
                    Download {label}
                  </span>
                  <span className="block font-mono text-[11px] opacity-80">
                    SHA-256 verified · {asset.size} · v{data.version}
                  </span>
                </span>
                <Icon
                  name="arrow_forward"
                  className="ml-2 text-base transition-transform group-hover:translate-x-1"
                />
              </a>
            )}
          </div>

          {asset && (
            <p className="text-xs text-on-surface-variant opacity-80 font-mono break-all mb-4">
              sha256: {asset.sha256}
            </p>
          )}

          <ol className="space-y-3 text-sm text-on-surface-variant">
            <li className="flex gap-3">
              <span className="text-primary-fixed-dim font-bold">1.</span>
              <span>
                {os === 'windows'
                  ? 'Run the downloaded .exe and approve the wizard (UAC only for System install).'
                  : os === 'macos'
                    ? 'Open the downloaded binary. First launch: right-click → Open if Gatekeeper blocks it.'
                    : 'Save the file, then chmod +x and run it (e.g. chmod +x ven-setup-linux-x64 && ./ven-setup-linux-x64).'}
              </span>
            </li>
            <li className="flex gap-3">
              <span className="text-primary-fixed-dim font-bold">2.</span>
              <span>Open a new terminal and run <code className="text-on-surface">ven --version</code>.</span>
            </li>
            <li className="flex gap-3">
              <span className="text-primary-fixed-dim font-bold">3.</span>
              <span>
                Prefer scripts? Use the one-liner tabs below, or{' '}
                <code className="text-on-surface">ven-setup --cli</code> on servers without a display.
              </span>
            </li>
          </ol>
        </div>
        <div className="hidden md:flex w-48 aspect-square bg-surface-container-high rounded items-center justify-center border border-outline-variant/30">
          <Icon name="desktop_windows" className="text-[64px] text-primary-fixed-dim" />
        </div>
      </div>
    </div>
  )
}

// One-click corporate download CTA. Replaces the old "run these commands"
// callout. The visitor sees:
//
//   - A primary download button auto-targeted at their detected OS + sensible
//     default arch (so 90 % of users never have to think).
//   - Tabs to switch OS, and (when the chosen OS has multiple arches in this
//     release) a small arch toggle.
//   - A 3-step "Download → Extract → Double-click <shim>" list naming the
//     exact shim filename for the chosen OS so there is zero command-line
//     instruction in the happy path.
//   - An "Advanced (skip the shim)" disclosure for power users who still
//     want to run `./ven-launcher` manually.
function CorporateDownload({ data, err }) {
  const [os, setOs] = useState(() => {
    const detected = detectPlatform()
    return OS_LABEL[detected] ? detected : 'windows'
  })
  const [arch, setArch] = useState(() => OS_DEFAULT_ARCH[os] ?? 'x64')
  const [showAdvanced, setShowAdvanced] = useState(false)

  // Whenever the OS tab changes, snap arch back to that OS's default so the
  // user never lands on a (windows, arm64) combo they didn't intend.
  function pickOs(next) {
    setOs(next)
    setArch(OS_DEFAULT_ARCH[next] ?? 'x64')
  }

  const asset = findLauncherAsset(data, os, arch)
  const shim = OS_SHIM[os]
  const archesAvailable = ARCH_ORDER.filter((a) => findLauncherAsset(data, os, a))

  return (
    <div className="glass-surface p-8 border-l-4 border-secondary-fixed-dim rounded-r-xl">
      <div className="flex flex-col md:flex-row gap-8 items-start">
        <div className="grow w-full">
          <div className="flex items-center gap-2 mb-2 flex-wrap">
            <span className="text-[10px] uppercase tracking-widest font-bold text-secondary-fixed-dim bg-secondary-fixed-dim/10 px-2 py-0.5 rounded">
              NEW IN v0.1.2
            </span>
            <span className="text-[10px] uppercase tracking-widest text-on-surface-variant opacity-70">
              works behind Zscaler · no admin · no PATH edits
            </span>
          </div>
          <h2 className="font-headline-md text-headline-md mb-2 text-secondary-fixed-dim">
            Corporate &amp; portable — one-click bundle
          </h2>
          <p className="text-on-surface-variant text-body-base mb-6">
            Locked-down laptop where{' '}
            <code className="text-on-surface">irm | iex</code> and{' '}
            <code className="text-on-surface">curl | sh</code> are blocked? Download the
            zip, extract anywhere, and double-click the bundled terminal shim. A shell
            opens with <code className="text-on-surface">ven</code> already activated —
            no command line typing, no admin prompt, no proxy issues.
          </p>

          {/* OS tabs */}
          <div className="flex flex-wrap gap-2 mb-3">
            {OS_ORDER.map((id) => (
              <button
                key={id}
                type="button"
                onClick={() => pickOs(id)}
                className={clsx(
                  'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-mono transition-colors border',
                  os === id
                    ? 'border-secondary-fixed-dim text-secondary-fixed-dim bg-secondary-fixed-dim/10'
                    : 'border-outline-variant/40 text-on-surface-variant hover:text-on-surface'
                )}
              >
                <Icon name={OS_ICON[id]} className="text-base" />
                {OS_LABEL[id]}
              </button>
            ))}
          </div>

          {/* Arch toggle (only shown when the OS has more than one arch) */}
          {archesAvailable.length > 1 && (
            <div className="flex items-center gap-2 mb-5 text-xs">
              <span className="text-on-surface-variant opacity-70 uppercase tracking-widest">
                arch
              </span>
              {archesAvailable.map((a) => (
                <button
                  key={a}
                  type="button"
                  onClick={() => setArch(a)}
                  className={clsx(
                    'px-2.5 py-0.5 rounded-full font-mono uppercase tracking-widest border transition-colors',
                    arch === a
                      ? 'border-secondary-fixed-dim text-secondary-fixed-dim'
                      : 'border-outline-variant/30 text-on-surface-variant hover:text-on-surface'
                  )}
                >
                  {a}
                </button>
              ))}
            </div>
          )}

          {/* Big download button */}
          <div className="mb-6">
            {err ? (
              <p className="text-sm text-on-surface-variant opacity-70">
                Releases manifest unavailable — see{' '}
                <a
                  className="text-primary-fixed-dim hover:underline"
                  href={RELEASES_URL}
                >
                  GitHub Releases
                </a>
                .
              </p>
            ) : !data ? (
              <p className="text-sm text-on-surface-variant opacity-70">
                Loading release assets…
              </p>
            ) : !asset ? (
              <p className="text-sm text-on-surface-variant opacity-70">
                No bundle for {OS_LABEL[os]} {arch} in v{data.version} — try a
                different arch above or browse{' '}
                <a className="text-primary-fixed-dim hover:underline" href={data.notesUrl}>
                  the release page
                </a>
                .
              </p>
            ) : (
              <a
                href={asset.url}
                className="group inline-flex items-center gap-3 px-6 py-4 rounded-xl bg-secondary-fixed-dim text-on-secondary-fixed font-bold shadow-lg hover:shadow-xl hover:-translate-y-0.5 transition-all"
              >
                <Icon name="download" className="text-2xl" />
                <span className="text-left">
                  <span className="block text-base">
                    Download for {OS_LABEL[os]} ({arch})
                  </span>
                  <span className="block font-mono text-[11px] opacity-80">
                    {asset.file} · {asset.size} · v{data.version}
                  </span>
                </span>
                <Icon
                  name="arrow_forward"
                  className="ml-2 text-base transition-transform group-hover:translate-x-1"
                />
              </a>
            )}
          </div>

          {/* 3-step "what to do after download" */}
          <ol className="space-y-3 mb-6">
            {[
              {
                n: 1,
                t: 'Extract the zip',
                b: 'Right-click → Extract All (Windows) or double-click (macOS / GNOME). Drop it on Desktop, in Downloads, on a USB stick — anywhere you have write access. No admin needed.',
              },
              {
                n: 2,
                t: (
                  <>
                    Double-click <code className="text-on-surface">{shim}</code>
                  </>
                ),
                b:
                  os === 'linux'
                    ? 'Most Linux file managers offer a "Run in Terminal" option for executable scripts. If yours does not, open a terminal in the extracted folder and run ./start-ven.sh.'
                    : os === 'macos'
                    ? 'Finder treats .command files as double-clickable Terminal scripts. The first launch may show a Gatekeeper warning — right-click → Open the first time and Gatekeeper remembers your choice.'
                    : 'Windows runs .cmd files without any execution-policy or admin gate. Double-click in Explorer and a shell opens.',
              },
              {
                n: 3,
                t: 'A terminal opens with ven activated',
                b: (
                  <>
                    Try <code className="text-on-surface">ven --version</code>,{' '}
                    <code className="text-on-surface">ven init</code>, or{' '}
                    <code className="text-on-surface">ven install node 22</code>. Close
                    the window when done — nothing was added to your system PATH or
                    shell rc files.
                  </>
                ),
              },
            ].map((s) => (
              <li key={s.n} className="flex gap-4 items-start">
                <span className="shrink-0 w-7 h-7 rounded-full border border-secondary-fixed-dim flex items-center justify-center text-xs font-bold text-secondary-fixed-dim">
                  {s.n}
                </span>
                <div className="grow text-sm">
                  <p className="font-bold text-on-surface mb-1">{s.t}</p>
                  <p className="text-on-surface-variant leading-relaxed">{s.b}</p>
                </div>
              </li>
            ))}
          </ol>

          {/* Optional: USB-stick / advanced toggle */}
          <button
            type="button"
            onClick={() => setShowAdvanced((v) => !v)}
            className="text-xs text-on-surface-variant hover:text-secondary-fixed-dim transition-colors flex items-center gap-1"
          >
            <Icon
              name={showAdvanced ? 'expand_less' : 'expand_more'}
              className="text-sm"
            />
            Advanced — USB-stick mode &amp; raw launcher invocation
          </button>
          {showAdvanced && (
            <div className="mt-3 space-y-3 text-sm text-on-surface-variant">
              <p>
                Want everything (runtimes, cache, lockfile state) inside the bundle so
                the same folder is portable across machines? Create a{' '}
                <code className="text-on-surface">.ven/</code> folder next to the
                launcher and ven will resolve <code>VEN_HOME</code> to it
                automatically.
              </p>
              <CodeBlock
                code={
                  os === 'windows'
                    ? 'mkdir .ven\n.\\ven-launcher.exe --show-env'
                    : 'mkdir .ven\n./ven-launcher --show-env'
                }
                prompt={os === 'windows' ? '>' : '$'}
                tone="success"
                copyable={false}
              />
              <p className="text-xs opacity-70">
                Power users can skip the shim entirely — call{' '}
                <code className="text-on-surface">
                  {os === 'windows' ? '.\\ven-launcher.exe' : './ven-launcher'}
                </code>{' '}
                directly. The shim is just a 3–7 line wrapper that does this for you.
              </p>
            </div>
          )}
        </div>
        <div className="hidden md:flex w-48 aspect-square bg-surface-container-high rounded items-center justify-center border border-outline-variant/30">
          <Icon name="business_center" className="text-[64px] text-secondary-fixed-dim" />
        </div>
      </div>
    </div>
  )
}

export default function Install() {
  // Lazy initializer — `detectPlatform` handles missing-navigator gracefully,
  // so we can compute the default tab without an effect-induced flash.
  const [active, setActive] = useState(() => detectPlatform())
  // Single fetch shared by Corporate CTA + Direct downloads table below.
  const { data: releases, err: releasesErr } = useReleasesManifest()

  const current = INSTALL[active] ?? INSTALL.windows

  return (
    <div className="max-w-[860px] mx-auto px-margin-mobile md:px-0 py-16">
      <Reveal as="header" className="text-center mb-16">
        <h1 className="font-display-lg text-display-lg mb-4 text-primary">Install ven</h1>
        <p className="text-on-surface-variant text-body-base max-w-md mx-auto">
          Download the GUI installer for your OS, or use a one-liner. SHA-256-verified end-to-end.
        </p>
      </Reveal>

      <Reveal as="section" className="mb-12">
        <InstallerDownload data={releases} err={releasesErr} />
      </Reveal>

      <PlatformTabs active={active} onChange={setActive} />

      <Reveal as="section" className="mb-20">
        <Terminal title="install command" bodyClassName="bg-surface-container-lowest">
          <code className="font-mono text-lg md:text-2xl text-primary-fixed-dim block">
            {current.cmd}
          </code>
        </Terminal>
        <p className="mt-4 text-center text-on-surface-variant font-mono text-xs opacity-60">
          {current.note}
        </p>
      </Reveal>

      <Reveal as="section" className="mb-24">
        <h2 className="font-headline-md text-headline-md mb-8 flex items-center gap-3">
          <Icon name="account_tree" className="text-primary-fixed-dim" />
          What gets installed
        </h2>
        <GlassCard tone="neutral" className="p-6 font-mono text-on-surface-variant leading-relaxed">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-primary-fixed-dim">$VEN_HOME/</span>
            <span className="text-[10px] bg-outline-variant px-1 text-on-surface uppercase font-bold">
              ROOT
            </span>
            <span className="text-[10px] text-on-surface-variant opacity-70 ml-1">
              defaults to ~/.ven · resolves to a sibling .ven/ for portable bundles
            </span>
          </div>
          <div className="pl-4 border-l border-outline-variant/30 py-1">
            <Row arrow="├──" name="bin/" tag="THE BINARIES" />
            <div className="pl-6 border-l border-outline-variant/30 ml-4 py-1 space-y-1">
              <Row arrow="├──" name="ven" inline tag="day-to-day CLI" />
              <Row arrow="├──" name="ven-launcher" inline tag="portable no-admin launcher" />
              <Row arrow="└──" name="ven-setup" inline tag="user / system installer" />
            </div>
            <Row arrow="├──" name="<runtime>/<version>/" tag="managed SDKs (node, python, …)" />
            <Row arrow="├──" name="cache/" tag="OSV / EOL / docs (SQLite)" />
            <Row arrow="└──" name="storage/" tag="lockfile simulations + drift state" />
          </div>
          <p className="mt-4 text-[11px] text-on-surface-variant opacity-70 leading-relaxed">
            Resolution order on every run:{' '}
            <code className="text-on-surface">$VEN_HOME</code> →{' '}
            <code className="text-on-surface">$VEN_STORAGE_PATH</code> →{' '}
            <code className="text-on-surface">&lt;launcher-dir&gt;/.ven</code> →{' '}
            <code className="text-on-surface">~/.ven</code>. The launcher exports the resolved value to
            every spawned shell so portable bundles stay self-contained.
          </p>
        </GlassCard>
      </Reveal>

      <Reveal as="section" className="mb-24 space-y-12">
        <h2 className="font-headline-md text-headline-md mb-4">Next steps</h2>
        <div className="grid gap-8">
          {[
            {
              n: 1,
              title: 'Install the shell hook',
              body: 'Adds the auto-activation hook to your shell profile. After this, `cd` into any project with a ven.toml and the runtime swaps automatically.',
              cmd: 'ven shell install',
              tone: 'border-primary-fixed-dim',
            },
            {
              n: 2,
              title: 'Install a runtime',
              body: 'Pick a language, pick a version. ven verifies the SHA-256 and runs a binary smoke-test before linking it under ~/.ven/<lang>/<version>/.',
              cmd: 'ven install node 22',
              tone: 'border-secondary-fixed-dim',
            },
            {
              n: 3,
              title: 'Bootstrap a project',
              body: 'Interactive runtime + package selection. Writes ven.toml. For Python, creates `./venv` and routes pip into it automatically.',
              cmd: 'ven init',
              tone: 'border-primary-fixed-dim',
            },
          ].map((s) => (
            <div key={s.n} className="flex gap-6">
              <div className="shrink-0 w-8 h-8 rounded-full border border-primary-fixed-dim flex items-center justify-center font-bold text-primary-fixed-dim">
                {s.n}
              </div>
              <div className="grow">
                <h3 className="font-headline-md text-xl mb-3">{s.title}</h3>
                <p className="text-on-surface-variant mb-4">{s.body}</p>
                <div
                  className={clsx(
                    'glass-surface bg-surface-container-lowest p-3 font-mono text-sm border-l-4',
                    s.tone
                  )}
                >
                  {s.cmd}
                </div>
              </div>
            </div>
          ))}
        </div>
      </Reveal>

      <Reveal as="section" className="mb-24">
        <CorporateDownload data={releases} err={releasesErr} />
      </Reveal>

      <Reveal as="section" className="mb-24">
        <h2 className="font-headline-md text-headline-md mb-8">Verify installation</h2>
        <div className="grid md:grid-cols-2 gap-4">
          <div className="glass-surface p-4 border border-outline-variant/20 rounded-xl">
            <p className="text-xs uppercase text-on-surface-variant opacity-50 mb-2 font-bold tracking-widest">
              Command
            </p>
            <code className="font-mono text-primary-fixed-dim block">ven --version</code>
          </div>
          <div className="glass-surface p-4 border border-outline-variant/20 rounded-xl">
            <p className="text-xs uppercase text-on-surface-variant opacity-50 mb-2 font-bold tracking-widest">
              Expected output
            </p>
            <code className="font-mono text-secondary-fixed-dim block">ven 0.2.0 (x86_64-pc-windows-msvc)</code>
          </div>
        </div>
      </Reveal>

      <Reveal as="section" className="mb-24">
        <h2 className="font-headline-md text-headline-md mb-8">Frequently asked</h2>
        <div className="grid md:grid-cols-2 gap-8">
          {FAQ.map((f) => (
            <div key={f.q}>
              <h3 className="font-bold text-primary-fixed-dim mb-2">{f.q}</h3>
              <p className="text-on-surface-variant text-sm leading-relaxed">{f.a}</p>
            </div>
          ))}
        </div>
      </Reveal>

      <Reveal as="section" className="mb-24 overflow-x-auto">
        <DownloadsTable data={releases} err={releasesErr} />
      </Reveal>

      <Reveal as="section" className="mb-24 border-t border-outline-variant/30 pt-16">
        <div className="max-w-3xl mx-auto">
          <header className="text-center mb-12">
            <span className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary-fixed-dim/10 text-primary-fixed-dim text-xs font-semibold tracking-wider uppercase mb-4">
              <Icon name="autorenew" size={14} />
              v0.1.7+ (ven CLI)
            </span>
            <h2 className="font-headline-md text-headline-md mb-4">Upgrade ven</h2>
            <p className="text-on-surface-variant">
              Already installed? Run <code className="text-on-surface">ven update</code>. It
              downloads the latest <code className="text-on-surface">ven</code> +{' '}
              <code className="text-on-surface">ven-launcher</code> from GitHub, verifies the
              SHA256 against the release manifest, and swaps them in place — no re-install,
              no PATH edits. System installs auto-elevate through UAC / <code className="text-on-surface">sudo</code>.
            </p>
          </header>

          <div className="space-y-6">
            <div>
              <header className="flex items-baseline gap-3 mb-3 flex-wrap">
                <h3 className="font-headline-md text-lg font-bold text-primary-fixed-dim">
                  Standard upgrade
                </h3>
                <span className="text-xs text-on-surface-variant opacity-70">
                  works on Windows / macOS / Linux, user + system installs
                </span>
              </header>
              <CodeBlock
                code={'ven update'}
                prompt="$"
                tone="cyan"
                language="shell"
              />
            </div>

            <div>
              <header className="flex items-baseline gap-3 mb-3 flex-wrap">
                <h3 className="font-headline-md text-lg font-bold text-primary-fixed-dim">
                  CI / scripted
                </h3>
                <span className="text-xs text-on-surface-variant opacity-70">
                  no confirmation prompt, machine-readable
                </span>
              </header>
              <CodeBlock
                code={'ven update --yes --json\nven update --check --json    # exit 0 even when current\nven update --version v0.1.6  # roll back to a specific tag'}
                prompt="$"
                tone="cyan"
                language="shell"
              />
            </div>
          </div>

          <div className="mt-8 rounded-lg border border-outline-variant/40 bg-surface-container-low/50 p-4">
            <p className="text-sm text-on-surface-variant">
              <strong className="text-on-surface">ven update</strong> vs{' '}
              <strong className="text-on-surface">ven upgrade</strong>:{' '}
              <code className="text-on-surface">ven update</code> updates the ven binaries
              themselves.{' '}
              <code className="text-on-surface">ven upgrade</code> updates the npm / pip /
              cargo / gem packages inside your project. Different commands, different
              surfaces.
            </p>
          </div>
        </div>
      </Reveal>

      <Reveal as="section" className="mb-24 border-t border-outline-variant/30 pt-16">
        <div className="max-w-3xl mx-auto">
          <header className="text-center mb-12">
            <h2 className="font-headline-md text-headline-md mb-4">Uninstall</h2>
            <p className="text-on-surface-variant">
              Leaving us? Since <span className="font-bold text-on-surface">v0.1.7</span> one
              command removes everything — binary, every installed runtime, cache, lockfile state,
              persisted <code className="text-on-surface">$VEN_HOME</code>, the pointer file, and
              the PATH entries the installer added. Honors a relocated storage root
              (<code className="text-on-surface">ven path set D:\ven</code> → uninstall removes
              both <code className="text-on-surface">~/.ven</code> AND
              <code className="text-on-surface"> D:\ven</code>).
            </p>
          </header>

          {/* The recommended path: one CLI command. */}
          <div className="mb-10">
            <header className="flex items-baseline gap-3 mb-3 flex-wrap">
              <h3 className="font-headline-md text-lg font-bold text-on-surface">
                {UNINSTALL.simple.label}
              </h3>
              <span className="text-xs text-on-surface-variant opacity-70">
                {UNINSTALL.simple.note}
              </span>
            </header>
            <CodeBlock
              code={UNINSTALL.simple.cmd}
              prompt={UNINSTALL.simple.prompt}
              tone="cyan"
              language="shell"
            />
            <p className="mt-3 text-xs text-on-surface-variant">
              Same teardown the bundled fallback scripts perform (
              <code className="text-on-surface">~/.ven/bin/ven-uninstall</code> on Unix,
              <code className="text-on-surface"> ~\.ven\bin\ven-uninstall.ps1</code> on Windows).
              System install detected? Re-run with{' '}
              <code className="text-on-surface">sudo</code> (Unix) or from an elevated PowerShell
              (Windows).{' '}
              <a
                href="https://github.com/bhuwanb23/ven/blob/main/docs/cmds/uninstall.md"
                target="_blank"
                rel="noreferrer"
                className="text-primary-fixed-dim hover:underline underline-offset-4"
              >
                Full docs →
              </a>
            </p>
          </div>

          {/* Escape hatch: copy-paste shell snippets for broken-install recovery. */}
          <details className="group border border-outline-variant/40 rounded-lg overflow-hidden">
            <summary className="cursor-pointer select-none px-4 py-3 text-sm font-semibold flex items-center justify-between bg-surface-container-low hover:bg-surface-container transition-colors">
              <span>
                Advanced: manual uninstall (no <code className="text-on-surface">ven</code> binary on PATH)
              </span>
              <span className="text-xs text-on-surface-variant opacity-60 group-open:hidden">
                show snippets ▼
              </span>
              <span className="text-xs text-on-surface-variant opacity-60 hidden group-open:inline">
                hide ▲
              </span>
            </summary>
            <div className="p-4 space-y-6 border-t border-outline-variant/30">
              <p className="text-xs text-on-surface-variant">
                Use these only when <code className="text-on-surface">ven uninstall</code> can't
                run — e.g. the binary is broken, missing from PATH, or you never installed it via
                the official script. Same logic as the bundled fallback scripts.
              </p>
              {UNINSTALL_ORDER.map((id) => {
                const u = UNINSTALL.advanced[id]
                return (
                  <div key={id}>
                    <header className="flex items-baseline gap-3 mb-3 flex-wrap">
                      <h4 className="font-headline-md text-base font-bold text-error">
                        {u.label}
                      </h4>
                      <span className="text-xs text-on-surface-variant opacity-70">
                        {u.note}
                      </span>
                    </header>
                    <CodeBlock
                      code={u.cmd}
                      prompt={u.prompt}
                      tone="cyan"
                      language={id === 'windows' ? 'powershell' : 'shell'}
                    />
                  </div>
                )
              })}
              <p className="text-xs text-on-surface-variant opacity-60">
                Using a non-default <code className="text-on-surface">$VEN_HOME</code> or a
                portable launcher with a sibling <code className="text-on-surface">.ven/</code>{' '}
                folder? Replace <code className="text-on-surface">~/.ven</code> /{' '}
                <code className="text-on-surface">%USERPROFILE%\.ven</code> with that path. The
                <code className="text-on-surface"> ven uninstall</code> CLI does this lookup
                automatically.
              </p>
            </div>
          </details>

          <div className="mt-6 text-sm text-center">
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noreferrer"
              className="text-primary-fixed-dim hover:underline underline-offset-4"
            >
              View source on GitHub →
            </a>
          </div>
        </div>
      </Reveal>
    </div>
  )
}

function Row({ arrow, name, tag, inline }) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-outline">{arrow}</span>
      <span className="text-on-surface">{name}</span>
      {tag && (
        <span
          className={clsx(
            inline ? 'text-[11px]' : 'text-[11px] font-bold',
            tag === 'THE BINARIES'
              ? 'text-secondary-fixed-dim'
              : 'opacity-40 italic'
          )}
        >
          {tag === 'THE BINARIES' ? `← ${tag}` : tag}
        </span>
      )}
    </div>
  )
}
