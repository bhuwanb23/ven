# ven v2 Promo — Expanded Production Prompt

## Title + Style Block

**Brand:** ven — Intelligent Version & Dependency Manager
**Palette (from design.md):**
- Surface: `#131313` (Void Black), `#0e0e0e` (container lowest)
- Container: `#201f1f`, outline variant: `#3a494b`
- On-surface: `#e5e2e1`, on-surface-variant: `#b9cacb`
- Primary/Electric Cyan: `#00f2ff`, fixed-dim: `#00dbe7`
- Secondary/Terminal Green: `#00ff41`, fixed-dim: `#00e639`
- Error/Conflict Red: `#ff3b30`, error: `#ffb4ab`
- Primary text (headlines): `#e1fdff`
- Outline: `#849495`

**Typography:** Geist (headings, UI) / JetBrains Mono (code, terminal, metadata)
**Aesthetic:** Dark glassmorphism, razor-sharp borders (2-4px video scale), cyanish glows, terminal-precision feel
**Corners:** 4px base (`rounded-sm`), 8px cards/terminals (`rounded-lg`)
**Spacing:** 4px grid, 80-140px video-scale padding

## Rhythm Declaration

```
HOOK-fast → PROBLEM-hold → REVEAL-slow → LANG-GRID-medium → AUTO-SWITCH-medium → GHOST-SCAN-build → CTA-hold→fade
```

7 scenes, 120 seconds total. Energy peaks at the Ghost Dependency scene (the "wow" feature). Calmest moment is the ven logo reveal (Scene 3). CTA is a warm close.

## Global Rules

- **3 layers minimum per scene:** BG texture (glow, ghost ring, dot grid, grain) + MG content + FG accents (rules, labels, dividers)
- **8-10 visual elements per scene,** 2+ decorative
- **6+ entrance tweens per scene** — staggered, 3+ different eases per scene
- **No exit animations except final scene** — transitions handle scene changes
- **All ambient pulses on the seekable tl**, never bare `gsap.to()`
- **Prefer `gsap.fromTo()` over `gsap.from()`** inside scene boundaries
- **Hard-kill every element after exit** with `tl.set(el, { opacity: 0, visibility: "hidden" }, t)`
- **No `<br>` tags** — use `max-width` for text wrapping
- **No `Math.random()`, `Date.now()`** — deterministic only
- **No `repeat: -1`** — calculate finite repeats from duration
- **Font sizes:** headlines 64-96px, body 28-42px, code 22-28px, labels 18-24px
- **Borders:** 2-4px, decorative opacity: 12-25%
- **Transitions:** mix of blur crossfade (connective), directional slide (topic change), zoom through (energy shift)

---

## Per-Scene Beats

### Scene 1: HOOK — The Chaos (0s–12s)

**Concept:** The viewer is thrown into the middle of a developer's worst day. Terminal windows stack and glitch — npm ERESOLVE, pip conflict, nvm not found, npx permission denied. Error messages flash red. The chaos builds to a peak, then cuts to black silence. Text over black: *"Every tool installs first."* Beat. *"Breaks second."*

**Mood:** Cyberpunk stress — think Blade Runner error terminals, but controlled. The errors should feel overwhelming but not amateurish.

**Depth layers:**
- BG: Dark `#131313` with a subtle dot-grid pattern (cyan dots at 5% opacity) that flickers
- BG: Multiple ghost terminal windows at 8% opacity, staggered and drifting slowly
- MG: Main terminal window — clean dark `#0e0e0e` border `#3a494b`, Mac-style traffic dots
- MG: Error lines type in one by one: `npm install express` → `ERESOLVE peer conflict`, `pip install django` → `dependency conflict`, `nvm use 18` → `version not installed`
- FG: Glitch effects on text (CSS RGB split via `text-shadow`), red highlights `#ff3b30` on error lines
- FG: A small monospace counter in bottom-right corner tracking "time wasted"

**Animation choreography:**
- Terminal window SLAMS in from top (`y: -200`, `expo.out`, 0.5s)
- Each command TYPES ON (opacity stagger, 0.25s each)
- Error lines FLASH in red (`opacity: 0→1` with `backgroundColor` flash)
- Ghost terminals DRIFT (ambient `y` oscillation on tl)
- Glitch STUTTERS on error text (tiny `skewX` pulse)
- End: everything HARD CUTS to black at 10.5s

**Transition out:** Blur through to black (0.3s), hold black 0.5s, then text fades in. Total transition: blur→black→hold→text.

---

### Scene 2: PROBLEM — The Reactive Trap (12s–26s)

**Concept:** A single terminal. A developer types with confidence. Everything seems fine until it isn't. The classic "install first, break later" workflow. We zoom into the moment of realization — the ERESOLVE error. Then the crushing truth: every tool is reactive.

**Mood:** Editorial/documentary moment. Clean, restrained. The calm after the chaos. Let the viewer absorb the message.

**Depth layers:**
- BG: `#131313` with a thin vertical accent line (Cyan `#00dbe7`, 4px) on the left edge, full height
- BG: Faint oversized ghost text "npm install" at 6% opacity, rotated slightly, bleeding off right edge
- MG: Terminal window (centered, 80% frame width) with traffic dots and path `~/projects/api`
- MG: Commands type in sequence: `npm install axios` → `✓ added 1`, `npm install express` → `✗ ERESOLVE peer dependency conflict`, `# 2 hours debugging...`
- FG: Label bottom-right "Reactive tools. Broken by design." in JetBrains Mono, cyan
- FG: A thin horizontal hairline rule (`#3a494b`) separates command area from error

**Animation choreography:**
- BG ghost text DRIFTS slowly (tl ambient, `x: 10`, 8s)
- Accent line EXPANDS from top (`scaleY: 0→1`, `transformOrigin: top`, 0.5s)
- Terminal window SLIDES in from bottom (`y: 80`, `power3.out`, 0.6s)
- Commands TYPE ON sequentially (opacity + a subtle width clip, staggered 0.3s)
- Error line CRASHES in with red glow flash (`color: #ff3b30`, 0.2s)
- Hairline rule DRAWS from center (`scaleX: 0→1`, 0.4s)
- Bottom label FADES in (0.4s)

**Transition out:** Blur crossfade (0.5s, `power2.inOut`) — the terminal blurs away as Scene 3 begins.

---

### Scene 3: REVEAL — Meet ven + Auto-Switch (26s–44s)

**Concept:** Silence. Black. Then a single cyan glow pulses at the center — the ven logo. It expands confidently. The brand statement locks in. Then we cut to the first demo: the auto-switch magic. `cd frontend` → Node 20 activates. `cd backend` → Node 22 + Python 3.11. No commands, no config — just `cd`. The viewer realizes: this tool *works while you think*.

**Mood:** Cinematic product reveal. The first half should feel like a luxury brand unveiling — controlled, confident, weighty. The second half shifts to "oh that's clever" energy.

**Depth layers:**
- BG: Pure `#131313` with a large radial cyan glow at center (`rgba(0, 219, 231, 0.15)`)
- BG: Ghost ring outline (`border: 3px solid rgba(0, 219, 231, 0.1)`, 380px) orbiting slowly
- MG: ven logo (from `Ven_logo.png`, 140px, circular with cyan border glow)
- MG: Headline "Meet ven" in Geist 88px `#e1fdff`, then subtitle "The Intelligent Version & Dependency Manager" 36px `#b9cacb`
- MG: After 6s hold, the scene transforms into a split-panel auto-switch demo
  - Left panel: file tree `~/projects/frontend/` highlighted
  - Right panel: terminal showing `cd frontend` → `node -v → v20.20.2`
- FG: Cyan accent line animates across the bottom third during the logo reveal

**Animation choreography:**
- Logo reveals with back.out (`scale: 0.3→1`, `opacity: 0→1`, 0.8s, `back.out(1.6)`)
- Headline FLOATS up from below (`y: 50→0`, `opacity: 0→1`, 0.6s, `power3.out`) at 0.6s
- Subtitle FLOATS up (0.5s, `power2.out`) at 0.9s
- After 4s hold, the auto-switch panel ASSEMBLES:
  - Left file tree SLIDES in from left (`x: -60`, 0.5s, `expo.out`)
  - Terminal panel SLIDES in from right (`x: 80`, 0.5s, `power3.out`)
  - `cd frontend` TYPES ON (0.3s), then `node -v` → `v20.20.2` appears (0.3s)
  - `cd backend` TYPES ON, then `node -v` → `v22.11.0` + `python --version` → `3.11.5` appear with green checkmarks
  - Green checkmark SNAPS in (`scale: 0→1`, `back.out(2)`, 0.3s)
- Ambient: logo BREATHES (tl `scale: 1→1.02`, 3s, `sine.inOut`, yoyo) 
- Ghost ring ORBITS (tl `rotation: 0→12`, 4s, `sine.inOut`, yoyo)

**Transition out:** Zoom-through (0.45s, `power3.in`→`expo.out`) — zoom past the ven logo into the language grid.

---

### Scene 4: LANGUAGES — 8 Runtimes, One Interface (44s–62s)

**Concept:** A clean 4×2 grid of language cards fills the frame. Each card shows the language name and the one-line install command. The pattern is hypnotic — same command structure, different language, every time. The message sinks in visually before you read a word: *one tool for everything.*

**Mood:** Clean, rhythmically satisfying. Like watching a perfectly organized tool rack. Satisfying, precise, confident.

**Depth layers:**
- BG: `#131313` with a faint cyan grid overlay (radial dots, 48px spacing, 6% opacity)
- BG: Oversized ghost text "ven install" at 5% opacity, repeating as a watermark pattern
- MG: Headline "8 languages. One interface." + subtitle "Same command. Every runtime."
- MG: 4×2 card grid with language cards
  - Each card: `#201f1f` bg, 2px `#3a494b` border, 8px radius
  - Language name (Geist, 32px, `#e5e2e1`), Command (JetBrains Mono, 20px, `#00a3ad`)
  - Cards: Node.js, Python, Go, Rust, Java, Deno, Bun, Ruby
- FG: Thin horizontal rule below headline (`#3a494b`, 1px)
- FG: Small version badge on each card "*" (indicating any version) in Terminal Green `#00e639`

**Animation choreography:**
- Headline SLIDES in from left (`x: -50`, 0.6s, `power3.out`)
- Subtitle FADES in (0.45s, `power2.out`) at 0.2s
- Rule EXPANDS from center (`scaleX: 0→1`, 0.4s) at 0.3s
- Language cards STAGGER in from below with `back.out(1.2)`: `y: 60→0`, `opacity: 0→1`, 0.45s each, stagger 0.08s
- Each card's border glow PULSES on hover (ambient: `border-color` shift on tl, staggered)
- Version badges SNAP in after cards (0.2s each, `back.out(2)`)
- BG grid DRIFTS subtly (tl, `backgroundPosition`, 8s, `none`)

**Transition out:** Push-slide up (0.4s, `power2.in`) — the grid slides upward and the auto-switch scene pushes in from below.

---

### Scene 5: AUTO-SWITCH DEEP DIVE + GHOST DEPENDENCY (62s–85s)

**Concept:** The most technically impressive scene. We show three things working together:
1. Multi-language project with `ven.toml` — Node + Python + Go all defined
2. The shell hook detecting `cd` and activating the right mix
3. Then `ven scan --ghosts` finding an undeclared import — the "ghost dependency" feature that no other tool has

This is where the developer watching goes "wait, that's actually useful."

**Mood:** Technical deep-dive but visually clear. Think Stripe's API docs — clean, focused, every pixel earns its place. The ghost detection moment should feel like a detective reveal.

**Depth layers:**
- BG: `#131313` with a subtle terminal scan-line overlay (horizontal lines at 4px spacing, 3% opacity)
- BG: Oversized ghost text of a `ven.toml` file at 6% opacity floating in background
- MG: Split panel:
  - Left (40%): `ven.toml` file shown with syntax highlighting style — `[runtime] node = "20"`, `python = "3.11"`, `go = "1.21"` — with a magnifying glass/scan effect passing over it
  - Right (60%): Terminal window showing the auto-switch + ghost scan sequence
- MG: Terminal scroll: `cd ~/project` → auto-activates Node 20, Python 3.11, Go 1.21 → `ven scan --ghosts` → finds `lodash` used but not declared → highlights it
- FG: A ghost icon 👻 (or a stylized "👻 GHOST DETECTED" chip) appears with a glow pulse at the moment of detection
- FG: Bottom stats bar showing before/after: "Declared: 12 pkgs | Ghosts: 3 | Fixed: ✓"

**Animation choreography:**
- Scan-line BG AMBIENT scrolls (tl `backgroundPosition`, 10s)
- Split panel ASSEMBLES: left panel SLIDES from left, right from right (both 0.5s, `expo.out`)
- `ven.toml` content TYPES ON line by line (0.25s each, staggered)
- Terminal shows auto-switch lines typing — green checkmarks SNAP for each runtime
- "ven scan --ghosts" command TYPES ON, pauses, then results FLASH in
- Ghost detection: a highlight sweep (CSS pattern) passes over the detected import
- Ghost icon POPS in with a glow burst (`scale: 0→1.2→1`, `back.out(2.5)`)
- Stats bar SLIDES up from bottom (0.4s)
- Green "Fixed" chip SNAPS in (0.3s, `back.out(2)`)

**Transition out:** Directional slide right (0.4s, `power3.inOut`) — as if moving to the next slide in a deck.

---

### Scene 6: SECURITY + TEAM SYNC (85s–104s)

**Concept:** The toolkit extends beyond individual productivity. Show `ven check` scanning for CVEs across 8 ecosystems. Show `ven lock` + `ven sync` for team reproducibility. The message: ven is production-ready, CI-safe, and enterprise-viable.

**Mood:** Trust, reliability, robustness. Lighter, more open than the dark terminal scenes — suggesting a team environment.

**Depth layers:**
- BG: `#131313` transitioning to a slightly warmer dark `#1a1a1a` with a subtle green-tinted radial glow from top-right
- BG: Hexagonal mesh pattern at 4% opacity (suggesting network/security topology)
- MG: Two side-by-side terminal windows (or one wide terminal with two phases)
  - Phase 1: `ven check` — CVE scan results showing packages with OK/CVE status
  - Phase 2: `ven lock` → `ven sync` — showing lockfile generation and team sync
- MG: Data visualization — a small horizontal bar chart showing CVE counts per ecosystem (npm, PyPI, Go, crates.io, Maven, RubyGems, Deno) all green "0 CRITICAL"
- FG: Shield/badge icon with "CI-SAFE" text in Terminal Green
- FG: Bottom: "Team sync" — two user avatars (stylized circles) connected by a line, with checkmark

**Animation choreography:**
- BG mesh DRIFTS subtly (tl, slight `x` + `y` oscillation, 12s)
- Shield icon FLOATS in from top-left (0.5s, `power3.out`)
- Terminal 1 SLIDES in (0.5s, `power3.out`)
- `ven check` commands TYPE ON — each CVE line ENTERS with green/red indicators
- Bar chart BARS GROW from left (`scaleX: 0→1`, staggered 0.15s, `power2.out`)
- Then Terminal 1 FADES out (via transition), Terminal 2 SLIDES in
- `ven lock` → `✓ ven.lock v2 written` types on
- `ven sync` → `✓ environment ready` with green check
- Team avatars ASSEMBLE (two circles CONNECT with a line that DRAWS)
- "CI-SAFE" badge SNAPS in bottom-right

**Transition out:** Slow dissolve (0.7s, `sine.inOut`) — winding down toward the close.

---

### Scene 7: CTA — Start Today (104s–120s)

**Concept:** The logo returns, centered and confident. The install command glows in a terminal block. Platform badges line up. The tagline delivers the final message. Then a slow, graceful fade to black. The viewer should feel: "I need this. And installing it is one command away."

**Mood:** Warm close. Confident but not aggressive. The developer should feel welcomed, not sold to.

**Depth layers:**
- BG: `#131313` with a large central cyan glow (`rgba(0, 219, 231, 0.12)`, 1000px)
- BG: Ghost ring (same as Scene 3) at larger scale, orbiting
- MG: ven logo (160px, prominent cyan glow border, `0 0 60px rgba(0, 219, 231, 0.3)`)
- MG: Headline "Start with ven today." (88px, `#e1fdff`)
- MG: Terminal block with install command: `curl -fsSL get.ven.sh | sh`
- MG: Platform badges: Windows · macOS · Linux in JetBrains Mono, `#00dbe7`
- FG: Tagline: "Install once. Switch automatically. Never break." (32px, `#b9cacb`)
- FG: Small text: "MIT License · github.com/bhuwanb23/ven" (22px, `#849495`)

**Animation choreography:**
- Logo REVEALS with `back.out(1.6)` (`scale: 0.3→1`, 0.8s)
- Headline FLOATS up (`y: 50→0`, 0.65s, `power3.out`) at 0.4s
- Terminal block DROPS in (`y: -40→0`, 0.5s, `expo.out`) at 0.7s
- Install command TYPES ON (0.4s) with a cyan cursor blink
- Platform badges FADE in (0.4s each, staggered 0.1s)
- Tagline FLOATS up (0.5s, `power2.out`)
- Small text FADES in (0.4s)
- Final fade: ALL elements fade out (`opacity: 0`, `y: -20`, 1.2s, `power2.in`, stagger 0.1s) starting at 116s
- Background fades to black (0.8s) at 118s
- `tl.set(#scene7, { visibility: "hidden" })` at 119s

**Transition out:** N/A — final scene, elements fade to black.

---

## Recurring Motifs

1. **The orbital ring** — appears in Scene 3 (logo reveal) and Scene 7 (CTA). The same ghost ring CSS, resized. Creates visual bookends.
2. **Terminal chrome** — every terminal uses the same Mac-style traffic dots + path bar + dark `#0e0e0e` bg with `#3a494b` border. Consistent across all scenes.
3. **Cyan accent line** — a 4px vertical line on the left edge, used in Scene 2 and Scene 5. Creates brand continuity.
4. **Green checkmark** — `back.out(2)` snap animation on every success state. Recognizable brand micro-interaction.
5. **Dot-grid background** — appears in Scene 1 (chaos) and Scene 4 (languages). Ties the energetic moments together.
6. **Ghost text** — oversized faded text as background texture, used in Scenes 2, 4, 5. Creates depth.

## Negative Prompt

- No Inter, Roboto, Open Sans, or Poppins — Geist + JetBrains Mono only
- No `<br>` in text — use `max-width` wrapping
- No full-screen linear gradients (H.264 banding) — use radial or solid + glow
- No `repeat: -1` on any tween
- No `position: absolute` for content containers — use flexbox + padding
- No `Math.random()`, `Date.now()`, or async timeline construction
- No `gsap.set()` on elements from future scenes — use `tl.set()` at the right time position
- No exit animations (except Scene 7) — transitions handle scene changes
- No full-screen solid #000 at any point (Scene 3 uses radial glow)
- No emoji that won't render — stick to Unicode symbols (✓, ✗, ⬡)
