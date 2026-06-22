// Per-language data driving both /languages and /docs/<slug>.
//
// Pinned versions match the values exercised by the verification matrix
// (`example/_run.ps1`) so the docs never advertise a toolchain that the
// release binary hasn't been smoke-tested against.

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
node = "22"

[packages]
express = "4.18.2"
lodash  = "*"`,
    env: [
      ['PATH', '~/.ven/node/22.22.2/bin'],
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
    versions: ['3.11', '3.12', '3.13'],
    pkgMgr: 'pip + venv',
    config: 'requirements.txt',
    status: 'stable',
    tagline: 'General-purpose language with pip for packages and venv for project isolation. ven creates ./venv on first `ven add` and routes pip into it — no manual activation.',
    install: ['ven install python 3.13', 'ven install python 3.12'],
    venToml: `[runtime]
python = "3.13"

[packages]
flask    = "3.0.0"
requests = "*"`,
    env: [
      ['PATH', '~/.ven/python/3.13.12/bin'],
      ['VIRTUAL_ENV', './venv'],
    ],
    packageOps: [
      ['ven add flask', './venv/Scripts/pip install flask'],
      ['ven remove requests', './venv/Scripts/pip uninstall requests'],
      ['ven upgrade django', './venv/Scripts/pip install --upgrade django'],
    ],
    includes: ['python', 'pip', 'venv'],
    downloads: 'python.org/downloads/',
  },
  {
    slug: 'go',
    name: 'Go',
    code: 'GO',
    versions: ['1.22', '1.24', '1.26'],
    pkgMgr: 'go mod',
    config: 'go.mod',
    status: 'stable',
    tagline: 'Compiled language from Google. ven manages Go versions and sets GOROOT. Package management is handled natively by go mod.',
    install: ['ven install go 1.26'],
    venToml: `[runtime]
go = "1.26"`,
    env: [
      ['PATH', '~/.ven/go/1.26.2/bin'],
      ['GOROOT', '~/.ven/go/1.26.2'],
      ['GOPATH', '~/go'],
    ],
    packageOps: [
      ['ven add github.com/google/uuid', 'go get github.com/google/uuid'],
      ['ven remove github.com/google/uuid', 'go mod tidy'],
    ],
    includes: ['go', 'gofmt', 'godoc'],
    downloads: 'go.dev/dl/',
  },
  {
    slug: 'rust',
    name: 'Rust',
    code: 'RS',
    versions: ['1.74', '1.75', '1.76'],
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
    versions: ['JDK 17', 'JDK 21', 'JDK 25'],
    pkgMgr: 'Maven / Gradle',
    config: 'pom.xml',
    status: 'stable',
    tagline: 'Enterprise-grade language. ven manages JDK versions and sets JAVA_HOME automatically. Package management via Maven or Gradle (user-managed).',
    install: ['ven install java 21', 'ven install java 17'],
    venToml: `[runtime]
java = "21"`,
    env: [
      ['PATH', '~/.ven/java/21.0.11+10.0.LTS/bin'],
      ['JAVA_HOME', '~/.ven/java/21.0.11+10.0.LTS'],
    ],
    packageOps: [
      ['ven add com.google.guava:guava', 'pom.xml + mvn install'],
      ['(native)', 'mvn install / gradle build'],
    ],
    includes: ['java', 'javac', 'jar', 'jshell'],
    downloads: 'adoptium.net',
  },
  {
    slug: 'ruby',
    name: 'Ruby',
    code: 'RB',
    versions: ['3.2', '3.3', '4.0'],
    pkgMgr: 'gem + bundler',
    config: 'Gemfile',
    status: 'stable',
    tagline: 'Dynamic language popular for web (Rails) and DevOps tooling. ven manages versions and uses gem/bundler for packages.',
    install: ['ven install ruby 4.0'],
    venToml: `[runtime]
ruby = "4.0"

[packages]
rails   = "7.1.0"
sinatra = "*"`,
    env: [
      ['PATH', '~/.ven/ruby/4.0.3/bin'],
      ['GEM_HOME', '~/.ven/ruby/4.0.3'],
      ['GEM_PATH', '~/.ven/ruby/4.0.3'],
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
    versions: ['2.6', '2.7'],
    pkgMgr: 'native / npm:',
    config: 'deno.json',
    status: 'stable',
    tagline: 'Modern JavaScript / TypeScript runtime by the creator of Node.js. Single binary, no package manager needed — imports via URL natively.',
    install: ['ven install deno 2.7'],
    venToml: `[runtime]
deno = "2.7"`,
    env: [
      ['PATH', '~/.ven/deno/2.7.14'],
      ['DENO_DIR', '~/.cache/deno'],
    ],
    packageOps: [
      ['ven add npm:chalk', 'deno add npm:chalk'],
      ['(native)', 'import x from "npm:express@4.18.2"'],
    ],
    includes: ['deno (single binary)'],
    downloads: 'github.com/denoland/deno/releases',
  },
  {
    slug: 'bun',
    name: 'Bun',
    code: 'BN',
    versions: ['1.1', '1.3'],
    pkgMgr: 'bun (npm-compatible)',
    config: 'package.json',
    status: 'stable',
    tagline: 'Fast all-in-one JavaScript runtime. Drop-in Node.js replacement. npm-compatible. Single binary like Deno, package.json like Node.',
    install: ['ven install bun 1.3'],
    venToml: `[runtime]
bun = "1.3"

[packages]
chalk   = "5.3.0"
lodash  = "*"`,
    env: [
      ['PATH', '~/.ven/bun/1.3.13'],
      ['BUN_INSTALL', '~/.ven/bun/1.3.13'],
    ],
    packageOps: [
      ['ven add chalk', 'bun add chalk'],
      ['ven remove lodash', 'bun remove lodash'],
      ['ven upgrade react', 'bun update react'],
    ],
    includes: ['bun — runtime + bundler + test runner'],
    downloads: 'github.com/oven-sh/bun/releases',
  },
  {
    slug: 'php',
    name: 'PHP',
    code: 'PH',
    versions: ['8.2', '8.3', '8.4'],
    pkgMgr: 'Composer',
    config: 'composer.json',
    status: 'stable',
    tagline: 'The most popular server-side language. ven manages PHP versions per-project and uses Composer for package operations.',
    install: ['ven install php 8.3', 'ven install php 8.4'],
    venToml: `[runtime]
php = "8.3"

[packages]
laravel = "*"`,
    env: [
      ['PATH', '~/.ven/php/8.3.x'],
      ['PHPRC', '~/.ven/php/8.3.x'],
    ],
    packageOps: [
      ['ven add laravel', 'composer require laravel/laravel'],
      ['ven remove laravel', 'composer remove laravel/laravel'],
      ['ven upgrade laravel', 'composer update laravel/laravel'],
    ],
    includes: ['php', 'composer'],
    downloads: 'php.net/downloads.php',
  },
]

export const COMING_SOON = [
  { name: 'Elixir', pkgMgr: 'Mix' },
  { name: '.NET', pkgMgr: 'NuGet' },
  { name: 'Zig', pkgMgr: 'Single binary' },
  { name: 'Lua', pkgMgr: 'LuaRocks' },
  { name: 'Swift', pkgMgr: 'Swift PM' },
  { name: 'Kotlin', pkgMgr: 'Gradle' },
  { name: 'Scala', pkgMgr: 'sbt' },
]

export const MOST_REQUESTED = [
  { name: 'Elixir', votes: 218, max: 400 },
  { name: '.NET', votes: 187, max: 400 },
  { name: 'Swift', votes: 143, max: 400 },
  { name: 'Zig', votes: 138, max: 400 },
]
