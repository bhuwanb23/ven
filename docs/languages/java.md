# Java in ven

`ven install java` pulls **Eclipse Adoptium Temurin** JDK builds via the Adoptium API. ven manages the JDK *runtime*, not your build dependencies — those continue to be managed by Maven or Gradle.

| Aspect                | Detail |
|-----------------------|--------|
| `ven.toml` key        | `runtime.java` |
| Install dir           | `~/.ven/java/<semver>/` |
| Source                | `https://api.adoptium.net/v3/assets/feature_releases/<feature>/ga` (vendor: `eclipse`, jvm_impl: `hotspot`, image_type: `jdk`) |
| Feature list scanned  | `8, 11, 17, 21, 22, 23` (for "latest" / picker) |
| Architectures         | Windows / Linux / macOS · `x64`, `aarch64` |
| Package manager       | None — use Maven / Gradle directly |
| Plugin                | `src/plugins/java.rs` |
| Downloader            | `src/core/java_install.rs` |

## Install

```bash
ven install java 21            # latest 21.x.x GA from Adoptium
ven install java 17.0.10       # exact (prefix match, picks the first 17.0.10*)
ven install java latest        # newest GA across the feature list
ven install java               # interactive picker
```

### What the installer actually does

1. Resolves your spec against the Adoptium release index, picks the first GA asset whose `version_data.semver` matches (prefix match for major-only specs).
2. Downloads the `binaries[0].package.link` (zip on Windows, tar.gz elsewhere) into `~/.ven/.cache/`.
3. Extracts into `~/.ven/java/<semver>/`, **stripping the first path component** (Temurin tarballs nest everything under e.g. `jdk-21+35-jre/`).
4. Verifies `bin/java[.exe]` exists.

The folder name is the **full semver** Adoptium reports, e.g. `21.0.3+9` (with the build metadata). Use that exact string in `ven.toml` if you want to pin tightly:

```toml
[runtime]
java = "21.0.3+9"
```

…or use a major like `"21"` and let activation pick the highest installed.

## Activation

```toml
[runtime]
java = "21"
```

When active:

| Variable           | Value                                                |
|--------------------|------------------------------------------------------|
| `PATH` (prepended) | `~/.ven/java/<v>/bin`                                |
| `VEN_JAVA_VERSION` | Resolved semver                                       |
| `JAVA_HOME`        | `~/.ven/java/<v>` (parent of `bin`) — required by Maven, Gradle, IntelliJ, etc. |

## Packages

There is **no `ven add` for Java**. The `cmd_add_java_notice` handler prints a hint and exits. Same for `ven remove` and `ven upgrade`. Use your build tool:

- **Maven:** edit `pom.xml`, run `mvn dependency:resolve` / `mvn install`.
- **Gradle:** edit `build.gradle` / `build.gradle.kts`, run `gradle build`.

The dependency-intelligence layer uses a stub adapter for Java — `ven check-add` is best-effort only.

### Configuration example

```toml
[runtime]
java = "21"

[env]
JAVA_OPTS = "-Xmx2g"
MAVEN_OPTS = "-Xmx1g"
```

## Tips

- **Multiple JDKs side-by-side:** install several majors (`ven install java 17` and `ven install java 21`) and switch by changing `runtime.java` in each project's `ven.toml`.
- **JRE-only?** Not supported by ven — `image_type=jdk` is hardcoded in the request. Adoptium publishes JREs, but ven doesn't expose them as a separate install path.
- **GraalVM, Azul Zulu, Liberica?** Not supported — ven currently asks only for `vendor=eclipse` (Temurin). Adding another vendor is a one-line change in `core/java_install.rs::resolve_download_link` (and a feature request).

## Common errors

| Symptom                                                                | Cause / fix                                                                  |
|------------------------------------------------------------------------|------------------------------------------------------------------------------|
| `No downloadable Java release found for <spec>`                        | Adoptium hasn't published a GA build matching that spec yet — try `latest`. |
| `Java <v> is not installed. Run: ven install java <v>`                 | Pin doesn't match `~/.ven/java/`. Remember the folder uses Adoptium's full semver including build metadata. |
| `Cannot reach adoptium`                                                | Network / proxy issue.                                                       |
| `mvn` / `gradle` not found                                             | ven doesn't ship those — install Maven/Gradle separately and put them on PATH (or use a wrapper like `mvnw`). |
