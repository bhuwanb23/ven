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
  GITHUB_URL,
  RELEASES_URL,
  detectPlatform,
} from '../content/site.js'

const FAQ = [
  {
    q: 'How fast is the auto-switching hook?',
    a: 'Sub-50ms on a warm cache. The shell hook is a single Rust binary call that reads `ven.toml`, resolves the runtime against `~/.ven`, and exports the new PATH — no shim, no fork-per-command tax.',
  },
  {
    q: 'Does it support Windows?',
    a: 'Yes — first-class. PowerShell 5.1 + 7+ hook, a portable `ven-launcher.exe` for locked-down corporate machines, and a UAC-aware `ven-setup.exe` for system installs.',
  },
  {
    q: 'Can it coexist with nvm / pyenv?',
    a: 'Yes, but auto-switching hooks will fight for PATH precedence. Recommended migration: install ven, run `ven status` to confirm your projects resolve, then remove the older tool to avoid double resolution.',
  },
  {
    q: 'Is there a GUI?',
    a: 'No — and there won\'t be. ven is CLI-first by design; every workflow is scriptable and CI-friendly. `ven status --json` is the structured surface for editor integrations.',
  },
  {
    q: 'How is it different from mise / asdf?',
    a: 'mise and asdf manage runtime versions only. ven adds a pre-install dependency graph engine, OSV-backed CVE scanning, endoflife.date EOL alerts, a unified `ven add` package surface across 8 ecosystems, deterministic ven.lock with SHA-256 content hash, and a portable launcher for restricted environments.',
  },
  {
    q: 'Where does ven store data?',
    a: 'Everything under `~/.ven/`. Binaries in `~/.ven/bin`, downloaded runtimes in `~/.ven/<lang>/<version>/`, and a SQLite cache at `~/.ven/cache/` for OSV / EOL / docs lookups.',
  },
]

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

function DownloadsTable() {
  const [data, setData] = useState(null)
  const [err, setErr] = useState(null)

  useEffect(() => {
    let cancelled = false
    // The manifest lives in `/public` so Vite serves it at the site root.
    // Fetching as JSON at runtime keeps the page release-cadence-aware
    // without needing a rebuild for every version bump.
    fetch('/releases-manifest.json')
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
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse font-mono text-sm">
          <thead>
            <tr className="border-b border-outline-variant/30 text-on-surface-variant uppercase text-[10px] tracking-widest">
              <th className="py-4 px-2">Platform</th>
              <th className="py-4 px-2">Artifact</th>
              <th className="py-4 px-2">SHA-256</th>
              <th className="py-4 px-2 text-right">Size</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-outline-variant/10">
            {data.assets.map((d) => (
              <tr key={d.file} className="hover:bg-surface-container-low transition-colors">
                <td className="py-4 px-2 font-bold text-primary-fixed-dim">{d.platform}</td>
                <td className="py-4 px-2">
                  <a
                    href={d.url}
                    target="_blank"
                    rel="noreferrer"
                    className="hover:text-primary-fixed-dim underline-offset-4 hover:underline"
                  >
                    {d.file}
                  </a>
                </td>
                <td className="py-4 px-2 opacity-60">{d.sha256}</td>
                <td className="py-4 px-2 text-right">{d.size}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="mt-6 text-xs text-on-surface-variant opacity-70">
        Every asset ships with a `.sha256` sidecar and an aggregate `SHA256SUMS` manifest. The install
        scripts verify hashes automatically before extraction.
      </p>
    </>
  )
}

export default function Install() {
  // Lazy initializer — `detectPlatform` handles missing-navigator gracefully,
  // so we can compute the default tab without an effect-induced flash.
  const [active, setActive] = useState(() => detectPlatform())

  const current = INSTALL[active] ?? INSTALL.windows

  return (
    <div className="max-w-[860px] mx-auto px-margin-mobile md:px-0 py-16">
      <Reveal as="header" className="text-center mb-16">
        <h1 className="font-display-lg text-display-lg mb-4 text-primary">Install ven</h1>
        <p className="text-on-surface-variant text-body-base max-w-md mx-auto">
          Zero dependencies. Native binary. One command. SHA-256-verified end-to-end.
        </p>
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
            <span className="text-primary-fixed-dim">~/.ven/</span>
            <span className="text-[10px] bg-outline-variant px-1 text-on-surface uppercase font-bold">
              HOME
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

      <Reveal as="section" className="mb-24 glass-surface p-8 border-l-4 border-secondary-fixed-dim rounded-r-xl">
        <div className="flex flex-col md:flex-row gap-8 items-start">
          <div className="grow">
            <h2 className="font-headline-md text-headline-md mb-4 text-secondary-fixed-dim">
              Corporate &amp; portable
            </h2>
            <p className="text-on-surface-variant text-body-base mb-6">
              Restricted environment? Run <code className="text-on-surface">ven-launcher.exe</code> from
              anywhere — a USB stick, Downloads, a network share. It spawns a shell with the project's
              ven.toml applied, writes nothing to disk, and leaves the host machine untouched on exit.
            </p>
            <CodeBlock code="./ven-launcher.exe" prompt="$" tone="success" copyable={false} />
          </div>
          <div className="w-full md:w-48 aspect-square bg-surface-container-high rounded flex items-center justify-center border border-outline-variant/30">
            <Icon name="business_center" className="text-[64px] text-secondary-fixed-dim" />
          </div>
        </div>
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
            <code className="font-mono text-secondary-fixed-dim block">ven 1.0.0 (x86_64-pc-windows-msvc)</code>
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
        <DownloadsTable />
      </Reveal>

      <Reveal as="section" className="mb-24 border-t border-outline-variant/30 pt-16">
        <div className="text-center max-w-lg mx-auto">
          <h2 className="font-headline-md text-headline-md mb-4">Uninstall</h2>
          <p className="text-on-surface-variant mb-8">
            Leaving us? Remove everything with a single sweep.
          </p>
          <div className="bg-surface-container-lowest p-4 font-mono text-on-error border border-error/30 inline-block rounded">
            rm -rf ~/.ven &amp;&amp; sed -i '/\.ven\/bin/d' ~/.bashrc ~/.zshrc
          </div>
          <p className="mt-6 text-xs text-on-surface-variant opacity-60">
            On Windows: <code className="text-on-surface">ven-setup --uninstall</code> reverses the PATH
            edit and removes <code className="text-on-surface">%USERPROFILE%\.ven</code>.
          </p>
          <div className="mt-8 text-sm">
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
