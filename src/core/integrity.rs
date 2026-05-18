//! Shared install-integrity helpers used by every language installer.
//!
//! Two responsibilities:
//!   1. SHA256 archive verification before extract.
//!   2. Post-install binary smoke test (`<bin> --version` succeeds and contains an
//!      expected substring) so we never declare an install successful when the
//!      extracted bytes can't actually run.

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Compute the SHA256 of `file` and compare it (case-insensitive) to
/// `expected_hex`. Returns `Ok(())` only on a byte-for-byte match.
pub fn verify_sha256(file: &Path, expected_hex: &str) -> Result<()> {
    let mut f = File::open(file)
        .with_context(|| format!("Could not open {} for hashing", file.display()))?;
    let mut buf = [0u8; 64 * 1024];
    let mut hasher = Sha256::new();
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual = format!("{:x}", hasher.finalize());
    let expected = expected_hex.trim().to_ascii_lowercase();
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "SHA256 mismatch for {}\n  expected: {}\n  actual:   {}",
            file.display(),
            expected,
            actual
        ))
    }
}

/// Fetch a remote sidecar URL (e.g. `<archive>.sha256`) and return the first
/// hex token. Many publishers emit either bare `<hex>` or `<hex>  <filename>`.
pub fn fetch_sidecar_sha256(url: &str) -> Result<String> {
    let body = Client::new()
        .get(url)
        .send()
        .with_context(|| format!("Could not fetch checksum sidecar {}", url))?
        .error_for_status()
        .with_context(|| format!("Checksum sidecar HTTP error {}", url))?
        .text()?;
    extract_first_hex_token(&body).ok_or_else(|| {
        anyhow!(
            "Checksum sidecar at {} did not contain a hex SHA256 token",
            url
        )
    })
}

/// Fetch a multi-file `SHA256SUMS`-style manifest at `url` and pick out the
/// 64-char hex hash for the row whose filename equals `archive_filename`.
pub fn fetch_manifest_sha256(url: &str, archive_filename: &str) -> Result<String> {
    let body = Client::new()
        .get(url)
        .send()
        .with_context(|| format!("Could not fetch checksum manifest {}", url))?
        .error_for_status()
        .with_context(|| format!("Checksum manifest HTTP error {}", url))?
        .text()?;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let hash = match parts.next() {
            Some(h) => h.trim_start_matches('*'),
            None => continue,
        };
        let name = parts.last().unwrap_or("").trim_start_matches('*');
        if name == archive_filename || line.contains(archive_filename) {
            if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(hash.to_string());
            }
        }
    }
    Err(anyhow!(
        "No SHA256 entry for {} in manifest {}",
        archive_filename,
        url
    ))
}

fn extract_first_hex_token(body: &str) -> Option<String> {
    for line in body.lines() {
        for tok in line.split(|c: char| !c.is_ascii_hexdigit()) {
            if tok.len() == 64 && tok.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(tok.to_ascii_lowercase());
            }
        }
    }
    None
}

/// Run `<bin> <args...>` and confirm the combined stdout+stderr contains
/// `expected_substr` (case-insensitive). Returns the trimmed first non-empty
/// line for the caller to print.
pub fn smoke_test_binary(bin: &Path, args: &[&str], expected_substr: &str) -> Result<String> {
    smoke_test_binary_with_env(bin, args, expected_substr, &[])
}

/// Variant of [`smoke_test_binary`] that lets the caller inject env vars into
/// the child process. Needed for Rust's `cargo` / `rustc`, which are rustup
/// shims that consult `RUSTUP_HOME` / `CARGO_HOME` to find the right
/// per-toolchain config — without those the shim falls back to the user's
/// global `~/.rustup`, finds no default toolchain matching ven's per-version
/// install, and exits with:
///
/// ```text
/// error: rustup could not choose a version of cargo to run,
///        because one wasn't specified explicitly,
///        and no default is configured.
/// ```
///
/// (See `src/core/rust_install.rs`. Other languages don't need this because
/// their binaries are self-contained, not shims.)
pub fn smoke_test_binary_with_env(
    bin: &Path,
    args: &[&str],
    expected_substr: &str,
    envs: &[(&str, &Path)],
) -> Result<String> {
    let mut cmd = Command::new(bin);
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd
        .output()
        .with_context(|| format!("Could not exec {} {:?}", bin.display(), args))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = format!("{stdout}\n{stderr}");

    if !output.status.success() {
        return Err(anyhow!(
            "Smoke test failed: {} {:?} exited with {:?}\n{}",
            bin.display(),
            args,
            output.status.code(),
            combined.trim()
        ));
    }
    if !combined
        .to_ascii_lowercase()
        .contains(&expected_substr.to_ascii_lowercase())
    {
        return Err(anyhow!(
            "Smoke test for {}: output did not contain {:?}\n{}",
            bin.display(),
            expected_substr,
            combined.trim()
        ));
    }
    let first = combined
        .lines()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    Ok(first)
}

/// Pretty-print a successful smoke test in the standard `[OK] verified: <line>` style.
pub fn print_smoke_ok(line: &str) {
    println!("  {} verified: {}", "[OK]".green(), line);
}

/// Pretty-print a successful checksum match.
pub fn print_checksum_ok(filename: &str) {
    println!("  {} checksum verified: {}", "[OK]".green(), filename);
}

/// Pretty-print a checksum-unavailable warning (matches existing Node UX).
pub fn print_checksum_unavailable(filename: &str, reason: &str) {
    println!(
        "  {} checksum unavailable for {} ({}). Continuing without verification.",
        "!".yellow(),
        filename,
        reason
    );
}

// ─────────────────────────────────────────────────────────────────────────
// HTTP client + streaming download (added in v0.1.6)
//
// Every language installer used to do this:
//
//     let resp = Client::new().get(url).send()?.error_for_status()?;
//     fs::write(&archive, resp.bytes()?)?;
//
// Two problems with that pattern:
//
//   1. `Client::new()` has no read-timeout configured. Behind Zscaler /
//      Netskope / Bluecoat the SSL inspector throttles bytes after the
//      handshake, the response body stalls, and reqwest eventually trips
//      with a confusing "error decoding response body / operation timed
//      out". The fix is to build a Client with explicit connect/read
//      timeouts so we fail fast and predictably.
//
//   2. `resp.bytes()?` buffers the entire archive into memory before a
//      single byte hits disk. RubyInstaller2 7z is ~30 MB, full Rust
//      toolchains 100+ MB — wasteful, and any progress bar fed from this
//      data is fake (the download is already finished by the time we
//      "stream" it). Streaming via `Response::read` gives a real progress
//      bar and bounded memory.
//
// `download_to_file` wraps both fixes and adds a 3-attempt retry loop for
// transient errors (timeouts, 5xx, connection resets) so a Zscaler hiccup
// doesn't kill an entire install.
// ─────────────────────────────────────────────────────────────────────────

/// Build a consistent installer user-agent like
/// `ven/0.1.6 (deno-installer)`. Use this for every download / API call
/// from an installer module so upstream rate-limit logs and analytics
/// can identify ven traffic distinctly from anonymous curl users.
pub fn installer_user_agent(language: &str) -> String {
    format!(
        "ven/{} ({}-installer)",
        env!("CARGO_PKG_VERSION"),
        language
    )
}

/// Maximum gap between two successful chunk reads before we declare the
/// stream stalled. Enforced in [`download_to_file_once`] (see comment
/// there); we can't enforce this at the reqwest level because
/// `reqwest::blocking::ClientBuilder` in 0.12.x doesn't expose
/// `read_timeout` — only `timeout` (total) and `connect_timeout`.
const READ_STALL_TIMEOUT: Duration = Duration::from_secs(60);

/// Hard total-request cap, used as a backstop if the body is just very
/// slow but never quite stalls. Big enough for any reasonable toolchain
/// (Go and Java tarballs are ~150–180 MB; at a corporate-throttled 100
/// KB/s that's ~30 min) without leaving a wedged process running forever.
const TOTAL_REQUEST_TIMEOUT: Duration = Duration::from_secs(45 * 60);

/// HTTP client tuned for ven's installer workflow.
///
/// - **30s connect timeout** — catches DNS failures, dead routes, and
///   corporate proxies that swallow the connection request quickly.
/// - **45min total request timeout** — generous backstop so big toolchains
///   on slow links can still complete; mid-stream stalls are actually
///   caught earlier by the per-chunk watchdog in
///   [`download_to_file_once`].
/// - **`rustls-tls-native-roots`** is enabled in `Cargo.toml`, so this
///   client trusts the OS cert store and works behind MITM proxies
///   (the v0.1.3 Zscaler fix).
///
/// `user_agent` is also recorded so upstream rate-limit logs can identify
/// ven specifically rather than seeing anonymous traffic.
///
/// NOTE: We can't use reqwest's own `read_timeout` setting because the
/// blocking `ClientBuilder` in reqwest 0.12.28 doesn't expose it (that
/// method exists only on the async builder in this version). Per-chunk
/// stall detection is implemented manually in `download_to_file_once`.
pub fn http_client(user_agent: &str) -> Result<Client> {
    Client::builder()
        .user_agent(user_agent)
        .connect_timeout(Duration::from_secs(30))
        .timeout(TOTAL_REQUEST_TIMEOUT)
        .build()
        .context("Failed to build reqwest::blocking::Client")
}

/// Stream `url` to `dest` with a real progress bar and retry on transient
/// errors. Returns the byte count written.
///
/// Prefer this over `Client::new().get(url).send()?.bytes()?` everywhere
/// in the installer modules — see the comment block above for why.
pub fn download_to_file(url: &str, dest: &Path, user_agent: &str) -> Result<u64> {
    const MAX_ATTEMPTS: u32 = 3;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match download_to_file_once(url, dest, user_agent) {
            Ok(n) => return Ok(n),
            Err(e) if attempt < MAX_ATTEMPTS && is_transient_http_error(&e) => {
                let backoff = Duration::from_secs(1u64 << (attempt - 1)); // 1s, 2s
                eprintln!(
                    "  {} attempt {}/{} failed: {:#}",
                    "[warn]".yellow(),
                    attempt,
                    MAX_ATTEMPTS,
                    e
                );
                eprintln!("         retrying in {}s...", backoff.as_secs());
                std::thread::sleep(backoff);
                last_err = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("download_to_file: unreachable")))
}

fn download_to_file_once(url: &str, dest: &Path, user_agent: &str) -> Result<u64> {
    let client = http_client(user_agent)?;
    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to start download: {url}"))?
        .error_for_status()
        .with_context(|| format!("Upstream HTTP error for {url}"))?;

    let total = response.content_length();
    let label = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    // Print before the bar — if the server hangs without sending any bytes,
    // at least the user sees what we're attempting.
    println!("  {} Downloading {} ...", "[DL]".cyan(), label.bold());

    let pb = ProgressBar::new(total.unwrap_or(0));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "  [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Write to a temp sibling so a half-downloaded file can't be mistaken
    // for a complete archive on the next run.
    let tmp = dest.with_extension(format!(
        "{}.partial",
        dest.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("download")
    ));
    let file =
        File::create(&tmp).with_context(|| format!("Failed to create {}", tmp.display()))?;
    let mut writer = BufWriter::new(file);

    let mut buf = [0u8; 64 * 1024];
    let mut total_written: u64 = 0;
    // Per-chunk stall watchdog. We can't use reqwest's read_timeout
    // (not exposed on the blocking ClientBuilder in 0.12.28), so we
    // measure the wall-clock gap between two successful chunk reads
    // ourselves and bail out — with a transient-classifier-friendly
    // error message — if it exceeds READ_STALL_TIMEOUT. The 3-attempt
    // retry loop in download_to_file then gives the connection a
    // fresh chance, which is exactly the behaviour we want behind a
    // hiccup-y Zscaler / Netskope proxy.
    let mut last_read_at = Instant::now();
    loop {
        let n = response
            .read(&mut buf)
            .with_context(|| format!("Failed while reading body of {url}"))?;
        if n == 0 {
            break;
        }
        // Manual stall check between reads — see comment above.
        // (We check AFTER read returns rather than wrapping read in a
        // timer thread because reqwest's blocking Read already blocks
        // up to TOTAL_REQUEST_TIMEOUT; a chunk that took >60s means
        // the link is wedged enough to give up early on this attempt.)
        let elapsed = last_read_at.elapsed();
        if elapsed > READ_STALL_TIMEOUT {
            return Err(anyhow!(
                "Download stalled: no bytes received for {}s while reading {} (operation timed out)",
                elapsed.as_secs(),
                url
            ));
        }
        last_read_at = Instant::now();
        writer
            .write_all(&buf[..n])
            .with_context(|| format!("Failed writing to {}", tmp.display()))?;
        total_written += n as u64;
        pb.set_position(total_written);
    }
    writer
        .flush()
        .with_context(|| format!("Failed flushing {}", tmp.display()))?;
    drop(writer);

    pb.finish_and_clear();

    // Atomic rename into final place. If the program crashes between
    // download and rename, the next run sees no `dest` and re-downloads.
    std::fs::rename(&tmp, dest).with_context(|| {
        format!(
            "Failed to rename {} -> {}",
            tmp.display(),
            dest.display()
        )
    })?;
    Ok(total_written)
}

fn is_transient_http_error(err: &anyhow::Error) -> bool {
    // anyhow doesn't downcast to reqwest::Error here because we wrap with
    // `.with_context(...)` everywhere; pattern-match on the chained
    // message string instead. Cheap and good enough for retry classification.
    let s = format!("{err:#}").to_ascii_lowercase();
    s.contains("timed out")
        || s.contains("timeout")
        || s.contains("connection reset")
        || s.contains("connection refused")
        || s.contains("connection aborted")
        || s.contains("error decoding response body")
        || s.contains("broken pipe")
        || s.contains("dns")
        || s.contains("502 ")
        || s.contains("503 ")
        || s.contains("504 ")
        || s.contains("status: 502")
        || s.contains("status: 503")
        || s.contains("status: 504")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_hex_from_bare_sha256() {
        let body = "5b3c8b8e6b5b06d6f3c11d0a25e98a3b18d6e2a16cb15b8e3eaa6f8c4f9c2e1a";
        assert_eq!(
            extract_first_hex_token(body).unwrap(),
            "5b3c8b8e6b5b06d6f3c11d0a25e98a3b18d6e2a16cb15b8e3eaa6f8c4f9c2e1a"
        );
    }

    #[test]
    fn extracts_hex_with_filename() {
        let body = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  some-archive.tar.gz\n";
        assert!(extract_first_hex_token(body).is_some());
    }

    #[test]
    fn http_client_builds_with_timeouts() {
        // Just ensure the builder accepts our combination of settings.
        let _ = http_client("ven-test/0.0").expect("client should build");
    }

    #[test]
    fn transient_classifier_catches_the_ruby_zscaler_message() {
        // This is the exact pattern the user hit on Ruby install behind Zscaler.
        let e = anyhow!("error decoding response body")
            .context("Failed while reading body of https://example.test/ruby.7z");
        assert!(is_transient_http_error(&e), "got: {e:#}");
    }

    #[test]
    fn transient_classifier_catches_typical_network_phrases() {
        for phrase in [
            "operation timed out",
            "connection reset by peer",
            "Connection refused",
            "broken pipe",
            "dns error: failed to lookup",
            "status: 503",
        ] {
            let e = anyhow::anyhow!("{}", phrase);
            assert!(
                is_transient_http_error(&e),
                "expected {phrase:?} classified transient"
            );
        }
    }

    #[test]
    fn transient_classifier_does_not_retry_permanent_errors() {
        let e = anyhow::anyhow!("404 Not Found");
        assert!(!is_transient_http_error(&e));
        let e = anyhow::anyhow!("Checksum mismatch");
        assert!(!is_transient_http_error(&e));
    }
}
