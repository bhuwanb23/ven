import { useState } from 'react'
import clsx from 'clsx'
import Icon from '../components/ui/Icon.jsx'
import CodeBlock from '../components/ui/CodeBlock.jsx'
import GlassCard from '../components/ui/GlassCard.jsx'
import Terminal from '../components/ui/Terminal.jsx'

const PLATFORMS = [
  {
    id: 'macos',
    label: 'macOS',
    cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh',
    note: 'Installs to ~/.ven/bin/ven and adds it to PATH automatically.',
  },
  {
    id: 'linux',
    label: 'Linux',
    cmd: 'curl -fsSL https://get.ven.sh/install.sh | sh',
    note: 'Bash / Zsh / Fish supported. Installs to ~/.ven/bin/ven.',
  },
  {
    id: 'windows',
    label: 'Windows',
    cmd: 'irm https://get.ven.sh/install.ps1 | iex',
    note: 'PowerShell 5.1+ or 7+. Installs to %USERPROFILE%\\.ven\\bin.',
  },
  {
    id: 'source',
    label: 'From Source',
    cmd: 'git clone https://github.com/yourorg/ven && cd ven && cargo build --release',
    note: 'Requires the Rust toolchain (rustup.rs). Two-pass build embeds ven + launcher into ven-setup.',
  },
]

const FAQ = [
  {
    q: 'Why is it so fast?',
    a: 'ven is pure Rust with a deterministic version-resolver and a SQLite cache. Shell hooks fire in under 50ms — no shellcheck-killing shims.',
  },
  {
    q: 'Does it support Windows?',
    a: 'Yes. Native PowerShell installer (5.1 and 7+), a portable ven-launcher.exe for locked-down corporate machines, and a UAC-aware ven-setup.exe for system installs.',
  },
  {
    q: 'Can it coexist with nvm / pyenv?',
    a: 'Yes, but the auto-switching hooks will fight for PATH precedence. Recommended: migrate to ven and remove the others to avoid double resolution.',
  },
  {
    q: 'Is there a GUI?',
    a: 'No. ven is CLI-first by design — every workflow is scriptable and CI-friendly. `ven status --json` is the structured surface.',
  },
  {
    q: 'How is it different from mise / asdf?',
    a: 'ven adds a pre-install dependency graph engine, OSV-backed CVE scanning, EOL alerts, and a unified package management surface across 8 ecosystems. Other multi-runtime tools focus only on the version part.',
  },
  {
    q: 'Where does ven store data?',
    a: 'Everything in ~/.ven/. Binaries under ~/.ven/bin, downloaded runtimes under ~/.ven/<lang>/<version>/, and a SQLite cache for OSV/EOL/docs lookups.',
  },
]

const DOWNLOADS = [
  { platform: 'macOS (Apple Silicon)', file: 'ven-macos-arm64.tar.gz', sha: '7b2f4e91…', size: '4.4 MB' },
  { platform: 'macOS (Intel)', file: 'ven-macos-x64.tar.gz', sha: '8f92a11b…', size: '4.8 MB' },
  { platform: 'Linux (x64)', file: 'ven-linux-x64.tar.gz', sha: '33a109fc…', size: '5.1 MB' },
  { platform: 'Windows (x64)', file: 'ven-windows-x64.zip', sha: 'a1c84e02…', size: '5.4 MB' },
]

export default function Install() {
  const [active, setActive] = useState(PLATFORMS[0].id)
  const current = PLATFORMS.find((p) => p.id === active) ?? PLATFORMS[0]

  return (
    <div className="max-w-[860px] mx-auto px-margin-mobile md:px-0 py-16">
      <header className="text-center mb-16">
        <h1 className="font-display-lg text-display-lg mb-4 text-primary">Install ven</h1>
        <p className="text-on-surface-variant text-body-base max-w-md mx-auto">
          The high-performance version manager. Zero dependencies. Native binary. One command to rule them all.
        </p>
      </header>

      <div className="flex flex-wrap justify-center gap-2 mb-12">
        {PLATFORMS.map((p) => (
          <button
            key={p.id}
            type="button"
            onClick={() => setActive(p.id)}
            className={clsx(
              'px-6 py-2 glass-surface transition-all border-b-2',
              p.id === active
                ? 'border-primary-fixed-dim text-primary-fixed-dim font-bold'
                : 'border-transparent text-on-surface-variant hover:text-primary-fixed-dim'
            )}
          >
            {p.label}
          </button>
        ))}
      </div>

      <section className="mb-20">
        <Terminal title="Install Command" bodyClassName="bg-surface-container-lowest">
          <code className="font-mono text-lg md:text-2xl text-primary-fixed-dim block">
            {current.cmd}
          </code>
        </Terminal>
        <p className="mt-4 text-center text-on-surface-variant font-mono text-xs opacity-60">
          {current.note}
        </p>
      </section>

      <section className="mb-24">
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
              <Row arrow="└──" name="ven-setup" inline tag="optional installer" />
            </div>
            <Row arrow="├──" name="<runtime>/<version>/" tag="managed SDKs (node, python, …)" />
            <Row arrow="├──" name="cache/" tag="OSV / EOL / docs (SQLite)" />
            <Row arrow="└──" name="storage/" tag="lockfile simulations + drift state" />
          </div>
        </GlassCard>
      </section>

      <section className="mb-24 space-y-12">
        <h2 className="font-headline-md text-headline-md mb-4">Next steps</h2>
        <div className="grid gap-8">
          {[
            {
              n: 1,
              title: 'Initialize shell integration',
              body: 'Install the auto-activation hook. After this, cd into a project with ven.toml and the runtime swaps automatically.',
              cmd: 'ven setup',
              tone: 'border-primary-fixed-dim',
            },
            {
              n: 2,
              title: 'Install a runtime',
              body: 'Pick a language, pick a version. ven verifies the SHA256 and runs a binary smoke-test before activating it.',
              cmd: 'ven install node 20',
              tone: 'border-secondary-fixed-dim',
            },
            {
              n: 3,
              title: 'Create your first project',
              body: 'Interactive runtime + package selection. Writes ven.toml. Activates the env in the current shell.',
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
      </section>

      <section className="mb-24 glass-surface p-8 border-l-4 border-secondary-fixed-dim rounded-r-xl">
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
      </section>

      <section className="mb-24">
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
      </section>

      <section className="mb-24">
        <h2 className="font-headline-md text-headline-md mb-8">Frequently asked</h2>
        <div className="grid md:grid-cols-2 gap-8">
          {FAQ.map((f) => (
            <div key={f.q}>
              <h3 className="font-bold text-primary-fixed-dim mb-2">{f.q}</h3>
              <p className="text-on-surface-variant text-sm leading-relaxed">{f.a}</p>
            </div>
          ))}
        </div>
      </section>

      <section className="mb-24 overflow-x-auto">
        <h2 className="font-headline-md text-headline-md mb-8">Direct downloads (v1.0.0)</h2>
        <table className="w-full text-left border-collapse font-mono text-sm">
          <thead>
            <tr className="border-b border-outline-variant/30 text-on-surface-variant uppercase text-[10px] tracking-widest">
              <th className="py-4 px-2">Platform</th>
              <th className="py-4 px-2">Artifact</th>
              <th className="py-4 px-2">SHA256</th>
              <th className="py-4 px-2 text-right">Size</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-outline-variant/10">
            {DOWNLOADS.map((d) => (
              <tr key={d.file} className="hover:bg-surface-container-low transition-colors">
                <td className="py-4 px-2 font-bold text-primary-fixed-dim">{d.platform}</td>
                <td className="py-4 px-2">{d.file}</td>
                <td className="py-4 px-2 opacity-50">{d.sha}</td>
                <td className="py-4 px-2 text-right">{d.size}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="mt-6 text-xs text-on-surface-variant opacity-70">
          All artifacts ship with per-asset .sha256 sidecars and an aggregate SHA256SUMS manifest. The
          install scripts verify automatically.
        </p>
      </section>

      <section className="mb-24 border-t border-outline-variant/30 pt-16">
        <div className="text-center max-w-lg mx-auto">
          <h2 className="font-headline-md text-headline-md mb-4">Uninstall</h2>
          <p className="text-on-surface-variant mb-8">
            Leaving us? Remove everything with a single sweep.
          </p>
          <div className="bg-surface-container-lowest p-4 font-mono text-on-error border border-error/30 inline-block rounded">
            rm -rf ~/.ven &amp;&amp; sed -i '/\.ven\/bin/d' ~/.bashrc ~/.zshrc
          </div>
        </div>
      </section>
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
