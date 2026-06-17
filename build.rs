//! Build script for the `ven` workspace.
//!
//! Its sole job is to populate `OUT_DIR` with two opaque blobs that
//! `src/bin/setup/common.rs` embeds via `include_bytes!`:
//!
//! - `ven.bin`          ← copy of `target/<profile>/ven[.exe]`
//! - `ven-launcher.bin` ← copy of `target/<profile>/ven-launcher[.exe]`
//!
//! Important Cargo semantics: build scripts run **once**, **before** any
//! bin in this package compiles. On a clean checkout the `ven` /
//! `ven-launcher` artifacts therefore do not yet exist when this script
//! runs, so we write empty placeholders and rely on `cargo:rerun-if-changed`
//! to repopulate the blobs on the **next** invocation. The canonical
//! release flow is two passes:
//!
//! ```text
//! cargo build --release --bin ven --bin ven-launcher
//! cargo build --release --bin ven-setup
//! ```
//!
//! `ven-setup` itself transparently falls back to sibling files on disk
//! when an embedded blob is empty, so development workflows still work.

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));

    // OUT_DIR is target/<profile>/build/<crate-hash>/out → climb 3 levels to get target/<profile>/.
    let target_profile_dir: PathBuf = out_dir
        .ancestors()
        .nth(3)
        .expect("Cannot resolve target/<profile> from OUT_DIR")
        .to_path_buf();

    let (ven_name, launcher_name) = if cfg!(target_os = "windows") {
        ("ven.exe", "ven-launcher.exe")
    } else {
        ("ven", "ven-launcher")
    };

    let ven_src = target_profile_dir.join(ven_name);
    let launcher_src = target_profile_dir.join(launcher_name);
    let ven_dst = out_dir.join("ven.bin");
    let launcher_dst = out_dir.join("ven-launcher.bin");

    embed_or_stub(&ven_src, &ven_dst, "ven");
    embed_or_stub(&launcher_src, &launcher_dst, "ven-launcher");

    write_sha256(&ven_dst, &out_dir.join("ven.bin.sha256"));
    write_sha256(&launcher_dst, &out_dir.join("ven-launcher.bin.sha256"));

    // Windows-only: bake an `asInvoker` manifest into `ven-setup` so Windows'
    // Installer Detection heuristics don't auto-elevate every invocation just
    // because the filename contains "setup". The Rust installer's
    // `relaunch_elevated` path is the only thing that legitimately needs UAC.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_setup_manifest();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bin/setup/ven-setup.manifest");
    println!("cargo:rerun-if-changed={}", ven_src.display());
    println!("cargo:rerun-if-changed={}", launcher_src.display());
}

fn embed_setup_manifest() {
    // Only the MSVC toolchain supports /MANIFESTUAC; gnu builds skip silently.
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return;
    }
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("bin")
        .join("setup")
        .join("ven-setup.manifest");
    if !manifest.is_file() {
        println!(
            "cargo:warning=ven-setup: manifest not found at {} -- skipping asInvoker embed",
            manifest.display()
        );
        return;
    }
    // Apply linker args only to the `ven-setup` binary (per-bin scoping).
    println!(
        "cargo:rustc-link-arg-bin=ven-setup=/MANIFESTINPUT:{}",
        manifest.display()
    );
    println!("cargo:rustc-link-arg-bin=ven-setup=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-bin=ven-setup=/MANIFESTUAC:NO");
}

fn write_sha256(src: &Path, dst: &Path) {
    if !src.is_file() || fs::read(src).unwrap_or_default().is_empty() {
        fs::write(dst, b"").unwrap_or_else(|e| {
            panic!("Failed to write empty stub {}: {e}", dst.display())
        });
        return;
    }
    let data = fs::read(src)
        .unwrap_or_else(|e| panic!("Failed to read {} for hashing: {e}", src.display()));
    let hash = Sha256::digest(&data);
    let hex = hash.iter().map(|b| format!("{b:02x}")).collect::<String>();
    fs::write(dst, hex.as_bytes())
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", dst.display()));
}

fn embed_or_stub(src: &Path, dst: &Path, label: &str) {
    if src.is_file() {
        fs::copy(src, dst).unwrap_or_else(|e| {
            panic!("Failed to copy {} -> {}: {e}", src.display(), dst.display())
        });
    } else {
        fs::write(dst, b"")
            .unwrap_or_else(|e| panic!("Failed to write stub {}: {e}", dst.display()));
        println!(
            "cargo:warning=ven-setup: '{}' not found at {} -- embedded payload for {} is empty. \
             Rebuild after `cargo build --release --bin ven --bin ven-launcher`.",
            label,
            src.display(),
            label
        );
    }
}
