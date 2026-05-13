// Per-language data driving both /languages and /docs/<slug> for languages.
// Source: website_design/language.md and the main repo's docs/languages/.

export const LANGUAGES = [
  {
    slug: 'node',
    name: 'Node.js',
    code: 'JS',
    versions: ['18', '20', '22'],
    pkgMgr: 'npm',
    config: 'package.json',
    status: 'stable',
    tagline: 'The most widely used JavaScript runtime. ven manages Node versions per-project and uses npm for package operations.',
    install: ['ven install node 20', 'ven install node 22'],
    venToml: `[runtime]
node = "20"

[packages]
express = "4.18.2"
lodash  = "*"`,
    env: [
      ['PATH', '~/.ven/node/20.20.2/bin'],
    ],
    packageOps: [
      ['ven add express', 'npm install express'],
      ['ven remove lodash', 'npm uninstall lodash'],
      ['ven upgrade react', 'npm update react'],
    ],
    includes: ['node', 'npm', 'npx'],
    downloads: 'nodejs.org/dist/',
  },
  {
    slug: 'python',
    name: 'Python',
    code: 'PY',
    versions: ['3.10', '3.11', '3.12'],
    pkgMgr: 'pip + venv',
    config: 'requirements.txt',
    status: 'stable',
    tagline: 'General-purpose language with pip for packages and venv for project isolation. ven handles venv creation and activation automatically.',
    install: ['ven install python 3.11', 'ven install python 3.12'],
    venToml: `[runtime]
python = "3.11"

[packages]
flask    = "3.0.0"
requests = "*"

[venv]
path = ".venv"`,
    env: [
      ['PATH', '~/.ven/python/3.11.5/bin'],
      ['PYTHONHOME', '~/.ven/python/3.11.5'],
    ],
    packageOps: [
      ['ven add flask', 'pip install flask'],
      ['ven remove requests', 'pip uninstall requests'],
      ['ven upgrade django', 'pip install --upgrade django'],
    ],
    includes: ['python', 'pip', 'venv'],
    downloads: 'python.org/downloads/',
  },
  {
    slug: 'go',
    name: 'Go',
    code: 'GO',
    versions: ['1.20', '1.21', '1.22'],
    pkgMgr: 'go mod',
    config: 'go.mod',
    status: 'stable',
    tagline: 'Compiled language from Google. ven manages Go versions and sets GOROOT / GOPATH. Package management is handled natively by go mod.',
    install: ['ven install go 1.21'],
    venToml: `[runtime]
go = "1.21"`,
    env: [
      ['PATH', '~/.ven/go/1.21.5/bin'],
      ['GOROOT', '~/.ven/go/1.21.5'],
      ['GOPATH', '~/go'],
    ],
    packageOps: [
      ['(native)', 'go get github.com/gin-gonic/gin'],
      ['(native)', 'go mod tidy'],
    ],
    includes: ['go', 'gofmt', 'godoc'],
    downloads: 'go.dev/dl/',
  },
  {
    slug: 'rust',
    name: 'Rust',
    code: 'RS',
    versions: ['1.73', '1.74', '1.75'],
    pkgMgr: 'cargo',
    config: 'Cargo.toml',
    status: 'stable',
    tagline: 'Systems language with Cargo as its all-in-one build system and package manager. ven manages Rust toolchain versions.',
    install: ['ven install rust 1.75'],
    venToml: `[runtime]
rust = "1.75"

[packages]
serde = "1.0"
tokio = "1.35"`,
    env: [
      ['PATH', '~/.ven/rust/1.75.0/bin'],
      ['CARGO_HOME', '~/.ven/rust/1.75.0'],
    ],
    packageOps: [
      ['ven add serde', 'cargo add serde'],
      ['ven remove tokio', 'cargo remove tokio'],
      ['ven upgrade serde', 'cargo update -p serde'],
    ],
    includes: ['rustc', 'cargo', 'rustfmt', 'clippy'],
    downloads: 'static.rust-lang.org',
  },
  {
    slug: 'java',
    name: 'Java',
    code: 'JV',
    versions: ['JDK 11', 'JDK 17', 'JDK 21'],
    pkgMgr: 'Maven / Gradle',
    config: 'pom.xml',
    status: 'stable',
    tagline: 'Enterprise-grade language. ven manages JDK versions and sets JAVA_HOME automatically. Package management via Maven or Gradle (user-managed).',
    install: ['ven install java 17', 'ven install java 21'],
    venToml: `[runtime]
java = "17"`,
    env: [
      ['PATH', '~/.ven/java/17.0.9/bin'],
      ['JAVA_HOME', '~/.ven/java/17.0.9'],
    ],
    packageOps: [
      ['(native)', 'mvn install / gradle build'],
    ],
    includes: ['java', 'javac', 'jar', 'jshell'],
    downloads: 'adoptium.net',
  },
  {
    slug: 'ruby',
    name: 'Ruby',
    code: 'RB',
    versions: ['3.1', '3.2', '3.3'],
    pkgMgr: 'gem + bundler',
    config: 'Gemfile',
    status: 'stable',
    tagline: 'Dynamic language popular for web (Rails) and DevOps tooling. ven manages versions and uses gem/bundler for packages.',
    install: ['ven install ruby 3.2'],
    venToml: `[runtime]
ruby = "3.2"

[packages]
rails   = "7.1.0"
sinatra = "*"`,
    env: [
      ['PATH', '~/.ven/ruby/3.2.2/bin'],
      ['GEM_HOME', '~/.ven/ruby/3.2.2'],
      ['GEM_PATH', '~/.ven/ruby/3.2.2'],
    ],
    packageOps: [
      ['ven add rails', 'gem install rails'],
      ['ven remove sinatra', 'gem uninstall sinatra'],
      ['ven upgrade rails', 'gem update rails'],
    ],
    includes: ['ruby', 'gem', 'bundler', 'irb'],
    downloads: 'rubyinstaller.org · ruby-lang.org',
  },
  {
    slug: 'deno',
    name: 'Deno',
    code: 'DN',
    versions: ['1.38', '1.39', '1.40'],
    pkgMgr: 'native / npm:',
    config: 'deno.json',
    status: 'stable',
    tagline: 'Modern JavaScript / TypeScript runtime by the creator of Node.js. Single binary, no package manager needed — imports via URL natively.',
    install: ['ven install deno 1.40'],
    venToml: `[runtime]
deno = "1.40"`,
    env: [
      ['PATH', '~/.ven/deno/1.40.0'],
      ['DENO_DIR', '~/.cache/deno'],
    ],
    packageOps: [
      ['(native)', 'deno.json import maps'],
      ['(native)', 'import x from "npm:express@4.18.2"'],
    ],
    includes: ['deno (single binary)'],
    downloads: 'github.com/denoland/deno/releases',
  },
  {
    slug: 'bun',
    name: 'Bun',
    code: 'BN',
    versions: ['1.0', '1.1'],
    pkgMgr: 'bun (npm-compatible)',
    config: 'package.json',
    status: 'stable',
    tagline: 'Fast all-in-one JavaScript runtime. Drop-in Node.js replacement. npm-compatible. Single binary like Deno, package.json like Node.',
    install: ['ven install bun 1.0'],
    venToml: `[runtime]
bun = "1.0"

[packages]
express = "4.18.2"
lodash  = "*"`,
    env: [
      ['PATH', '~/.ven/bun/1.0.20'],
      ['BUN_INSTALL', '~/.ven/bun/1.0.20'],
    ],
    packageOps: [
      ['ven add express', 'bun add express'],
      ['ven remove lodash', 'bun remove lodash'],
      ['ven upgrade react', 'bun update react'],
    ],
    includes: ['bun — runtime + bundler + test runner'],
    downloads: 'github.com/oven-sh/bun/releases',
  },
]

export const COMING_SOON = [
  { name: 'PHP', pkgMgr: 'Composer' },
  { name: 'Elixir', pkgMgr: 'Mix' },
  { name: '.NET', pkgMgr: 'NuGet' },
  { name: 'Zig', pkgMgr: 'Single binary' },
  { name: 'Lua', pkgMgr: 'LuaRocks' },
  { name: 'Swift', pkgMgr: 'Swift PM' },
  { name: 'Kotlin', pkgMgr: 'Gradle' },
  { name: 'Scala', pkgMgr: 'sbt' },
]

export const MOST_REQUESTED = [
  { name: 'PHP', votes: 342, max: 400 },
  { name: 'Elixir', votes: 218, max: 400 },
  { name: '.NET', votes: 187, max: 400 },
  { name: 'Swift', votes: 143, max: 400 },
  { name: 'Zig', votes: 138, max: 400 },
]
