//! Shared install-integrity helpers used by every language installer.
//!
//! Two responsibilities:
//!   1. SHA256 archive verification before extract.
//!   2. Post-install binary smoke test (`<bin> --version` succeeds and contains an
//!      expected substring) so we never declare an install successful when the
//!      extracted bytes can't actually run.

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

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
    let output = Command::new(bin)
        .args(args)
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
}
