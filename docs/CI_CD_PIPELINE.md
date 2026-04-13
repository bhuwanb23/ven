# CI/CD Pipeline Documentation

Comprehensive continuous integration and deployment pipeline for the ven project.

## 📊 Pipeline Overview

The CI/CD pipeline consists of **7 stages** that run automatically on every push and pull request:

```
Push/PR → Lint & Format → Test → Build → Security → Docs → (Release)
                                              ↓
                                          Coverage (Optional)
```

---

## 🎯 Pipeline Stages

### Stage 1: 🔍 Lint & Format

**Purpose:** Ensure code quality and consistency

**Checks:**
- ✅ Code formatting (`cargo fmt`)
- ✅ Clippy lints (`cargo clippy -- -D warnings`)
- ✅ Unused dependencies (cargo-machete)

**Triggers:** All pushes and PRs

**Failure Impact:** ❌ Blocks merge

---

### Stage 2: 🧪 Test

**Purpose:** Verify functionality across all platforms

**Platforms:**
- ✅ Ubuntu Linux
- ✅ Windows
- ✅ macOS

**Test Types:**
- Unit tests (`cargo test --lib`)
- Integration tests (`cargo test --test '*'`)
- Documentation tests (`cargo test --doc`)

**Failure Impact:** ❌ Blocks merge

---

### Stage 3: 📦 Build

**Purpose:** Ensure code compiles on all target platforms

**Builds:**
- Debug build (fast, with debug symbols)
- Release build (optimized, stripped)

**Targets:**
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `x86_64-apple-darwin`

**Artifacts:** Binaries uploaded for 7 days

**Failure Impact:** ❌ Blocks merge

---

### Stage 4: 🔒 Security Audit

**Purpose:** Identify security vulnerabilities in dependencies

**Checks:**
- ✅ Known vulnerabilities (cargo-audit)
- ⚠️ Outdated dependencies (cargo-outdated)

**Failure Impact:** ⚠️ Warning only (doesn't block)

---

### Stage 5: 📚 Documentation

**Purpose:** Build and verify API documentation

**Steps:**
- Generate rustdoc documentation
- Verify documentation structure
- Upload for review

**Artifacts:** API docs uploaded for 14 days

**Failure Impact:** ⚠️ Warning only

---

### Stage 6: 🚀 Release

**Purpose:** Automatically create GitHub releases with binaries

**Triggers:** Only on git tag push (e.g., `v0.1.0`)

**Requirements:** All previous stages must pass

**Actions:**
- Build release binaries for all platforms
- Create compressed archives
- Upload to GitHub Releases
- Generate release notes

**Artifacts:**
- `ven-v0.1.0-linux.tar.gz`
- `ven-v0.1.0-windows.zip`
- `ven-v0.1.0-macos.tar.gz`

**Failure Impact:** ❌ Blocks release

---

### Stage 7: 📊 Code Coverage (Optional)

**Purpose:** Track test coverage percentage

**Tool:** cargo-tarpaulin

**Output:** Cobertura XML report uploaded to Codecov

**Failure Impact:** ⚠️ Informational only

---

## 🔄 Trigger Events

| Event | Branches | What Runs |
|-------|----------|-----------|
| Push | `main`, `develop` | Stages 1-5, 7 |
| Pull Request | `main` | Stages 1-5, 7 |
| Tag Push | `v*` | All stages (1-7) |

---

## ⚡ Caching Strategy

The pipeline uses intelligent caching to speed up builds:

```yaml
Cache:
  - Cargo registry (~/.cargo/registry)
  - Git databases (~/.cargo/git)
  - Build artifacts (target/)

Cache Key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

**Benefits:**
- ⚡ 50-70% faster builds after first run
- 💰 Reduced CI minutes usage
- 🔄 Consistent dependency versions

---

## 🎨 Matrix Strategy

Tests and builds run in parallel across platforms:

```
                    ┌─────────────────┐
                    │   Push/PR       │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────▼─────┐ ┌─────▼─────┐ ┌─────▼─────┐
        │  Ubuntu   │ │  Windows  │ │   macOS   │
        │  Tests    │ │  Tests    │ │  Tests    │
        └─────┬─────┘ └─────┬─────┘ └─────┬─────┘
              │              │              │
        ┌─────▼─────┐ ┌─────▼─────┐ ┌─────▼─────┐
        │  Ubuntu   │ │  Windows  │ │   macOS   │
        │  Build    │ │  Build    │ │  Build    │
        └───────────┘ └───────────┘ └───────────┘
```

**Total parallel jobs:** 6 (3 platforms × 2 stages)

---

## 📦 Release Process

### Creating a Release

```bash
# 1. Update version in Cargo.toml
# 2. Commit changes
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"

# 3. Create tag
git tag -a v0.2.0 -m "Release v0.2.0"

# 4. Push tag (triggers release pipeline)
git push origin main --tags
```

### What Happens Automatically

1. **Pipeline validates:**
   - All tests pass ✅
   - Code quality checks pass ✅
   - Security audit completes ✅
   - Builds succeed on all platforms ✅

2. **Artifacts created:**
   - Linux binary (tar.gz)
   - Windows binary (zip)
   - macOS binary (tar.gz)

3. **GitHub Release created:**
   - Tag: `v0.2.0`
   - Binaries attached
   - Auto-generated release notes
   - Published to releases page

---

## 🔧 Configuration

### Environment Variables

```yaml
env:
  CARGO_TERM_COLOR: always    # Colored output
  RUST_BACKTRACE: 1           # Full error traces
```

### Job Dependencies

```
lint-and-format ──┐
                  │
test ─────────────┤
                  │
build ────────────┼──→ release
                  │
security-audit ───┤
                  │
docs ─────────────┘

coverage (independent)
```

### Failure Handling

| Stage | Fail Fast | Continue on Error |
|-------|-----------|-------------------|
| Lint & Format | ✅ Yes | ❌ No |
| Test | ❌ No | ❌ No |
| Build | ❌ No | ❌ No |
| Security Audit | ❌ No | ✅ Yes |
| Docs | ❌ No | ❌ Yes |
| Release | ✅ Yes | ❌ No |
| Coverage | ❌ No | ✅ Yes |

---

## 📈 Monitoring

### Viewing Pipeline Status

1. **GitHub Actions Tab:**
   - URL: `https://github.com/<owner>/ven/actions`
   - Shows all workflow runs

2. **PR Checks:**
   - Status appears in pull request
   - Green checkmark = all passed
   - Red X = something failed

3. **Badge (optional):**
   ```markdown
   ![CI/CD](https://github.com/<owner>/ven/actions/workflows/ci.yml/badge.svg)
   ```

### Common Issues

#### ❌ Lint Failures

**Problem:** Clippy warnings or formatting issues

**Fix:**
```bash
# Auto-fix formatting
cargo fmt

# Fix clippy warnings
cargo clippy --fix

# Commit changes
git add .
git commit -m "fix: apply clippy suggestions"
```

#### ❌ Test Failures

**Problem:** Tests failing on specific platform

**Fix:**
```bash
# Run tests locally
cargo test --all

# Run specific test
cargo test test_name

# Check platform-specific issues
cargo test --target x86_64-pc-windows-msvc
```

#### ❌ Build Failures

**Problem:** Compilation errors

**Fix:**
```bash
# Build locally
cargo build

# Build release
cargo build --release

# Check for dependency issues
cargo update
```

#### ⚠️ Security Warnings

**Problem:** Vulnerable dependencies

**Fix:**
```bash
# View audit report
cargo audit

# Update dependencies
cargo update

# Check for newer versions
cargo outdated
```

---

## 🚀 Performance Optimization

### Cache Hit Rates

| Cache Type | Expected Hit Rate | Savings |
|------------|------------------|---------|
| Cargo registry | 95%+ | ~60 seconds |
| Build artifacts | 80%+ | ~45 seconds |
| Git databases | 90%+ | ~10 seconds |

### Parallelization

- Tests run on 3 platforms simultaneously
- Builds run in parallel with tests
- Total pipeline time: ~3-5 minutes (vs 10+ minutes sequential)

---

## 📋 Artifacts Retention

| Artifact | Retention | Purpose |
|----------|-----------|---------|
| Test results | 7 days | Debug failures |
| Debug binaries | 7 days | Testing builds |
| Release binaries | 7 days | Pre-release testing |
| API documentation | 14 days | Review docs |
| Coverage reports | Indefinite (Codecov) | Track coverage |

---

## 🔐 Security

### Secrets Management

- GitHub Token: Automatically provided
- No additional secrets required for CI
- Release uses `${{ secrets.GITHUB_TOKEN }}`

### Dependency Scanning

- **cargo-audit:** Checks for known CVEs
- **cargo-outdated:** Identifies outdated packages
- **Manual review:** PR reviewers check dependency changes

### Supply Chain Security

- All dependencies from crates.io (verified registry)
- Lock file committed (`Cargo.lock`)
- Reproducible builds guaranteed

---

## 📊 Metrics

### Pipeline Health

| Metric | Target | Current |
|--------|--------|---------|
| Success rate | > 95% | Tracking... |
| Average duration | < 5 min | Tracking... |
| Cache hit rate | > 80% | Tracking... |
| Test coverage | > 70% | Tracking... |

### Improving Metrics

- Add more tests to increase coverage
- Optimize slow tests
- Use `--jobs` flag for parallel test execution
- Profile build times with `cargo build --timings`

---

## 🛠️ Customization

### Adding New Platforms

```yaml
# Add to matrix
matrix:
  os: [ubuntu-latest, windows-latest, macos-latest]
  include:
    - os: ubuntu-latest
      target: aarch64-unknown-linux-gnu  # ARM Linux
```

### Adding New Checks

```yaml
# Add new job
new-check:
  name: 🔍 New Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: cargo your-new-command
```

### Conditional Execution

```yaml
# Only run on main branch
if: github.ref == 'refs/heads/main'

# Only run for tags
if: startsWith(github.ref, 'refs/tags/')

# Only run if previous jobs succeeded
needs: [test, build]
```

---

## 📚 Related Documentation

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo CI Guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Rust Toolchain Action](https://github.com/dtolnay/rust-toolchain)
- [Caching Dependencies](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)

---

## 🎯 Best Practices

1. **Always run locally first:**
   ```bash
   cargo fmt && cargo clippy && cargo test
   ```

2. **Keep dependencies updated:**
   ```bash
   cargo update
   cargo audit
   ```

3. **Monitor pipeline:**
   - Check Actions tab regularly
   - Fix failures immediately
   - Don't ignore warnings

4. **Use cache effectively:**
   - Commit `Cargo.lock`
   - Don't clear cache unnecessarily
   - Use specific cache keys

5. **Tag releases carefully:**
   - Test thoroughly before tagging
   - Use semantic versioning (`v0.1.0`)
   - Write meaningful release notes

---

## 🆘 Troubleshooting

### Pipeline Stuck

**Solution:** Cancel and re-run
```bash
# In GitHub Actions UI:
# 1. Go to Actions tab
# 2. Click on stuck workflow
# 3. Click "Cancel workflow"
# 4. Click "Re-run all jobs"
```

### Cache Not Working

**Solution:** Update cache key
```yaml
# Add timestamp to force refresh
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}-${{ github.run_id }}
```

### Timeout Issues

**Solution:** Increase timeout
```yaml
timeout-minutes: 30  # Default is 360
```

---

**Last Updated:** 2024-03-22
**Pipeline Version:** 2.0
**Maintainer:** ven Development Team
