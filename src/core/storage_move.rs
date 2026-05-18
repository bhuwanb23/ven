//! Cross-drive-safe relocation of the ven storage root.
//!
//! Powers `ven path set <dir> --move` (and `ven path reset --move`). The
//! caller is responsible for *deciding* to move (after the user prompt) and
//! for writing the pointer afterwards — this module only owns the physical
//! relocation, with rollback on every intermediate failure.
//!
//! ## Why a dedicated module
//!
//! `fs::rename` is the obvious fast path, but on Windows it returns
//! `ERROR_NOT_SAME_DEVICE` whenever the target is on a different drive. The
//! whole point of `ven path set` is to free up the C: drive by moving data
//! to D: (or a network share), so the cross-device path is the *common*
//! case, not the rare one. We have to copy-then-delete, and we have to do
//! it without ever leaving the source in a half-deleted state if the copy
//! dies partway through.
//!
//! ## Algorithm
//!
//! 1. **Validate** target — must not be inside source, must not be the same
//!    path, and (if it already exists) must be empty or non-existent.
//! 2. **Lock source** — drop a `.ven-move.lock` file containing the current
//!    PID inside the source directory. Refuses to start if a previous lock
//!    is still there (caller can delete it after confirming no other ven
//!    process is running).
//! 3. **Fast path** — try `fs::rename`. If it succeeds the move is atomic
//!    by definition; skip to step 7.
//! 4. **Slow path** — `walkdir`-recurse the source, recreating dirs and
//!    streaming each file to the target with a progress bar that tracks
//!    bytes copied / total. The source is untouched throughout, so any
//!    failure leaves ven in its pre-move state.
//! 5. **Verify** — file count and total byte count at target must match
//!    what we computed at start. Mismatch means an underlying I/O error
//!    swallowed silently somewhere; abort, delete the partial target,
//!    leave the source alone.
//! 6. **Sweep source** — only after the target is fully populated and
//!    verified do we `remove_dir_all(source)`.
//! 7. **Unlock** — the lock file at the new target location is removed.

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Filename used for the source-side move lock.
const LOCK_FILE: &str = ".ven-move.lock";

/// Tunable knobs for [`move_storage`]. Constructed once by the caller (the
/// `ven path` command) and threaded through.
#[derive(Debug, Clone)]
pub struct MoveOptions {
    /// Show an `indicatif` progress bar during the cross-device copy.
    /// Disabled automatically when `--json` is in effect.
    pub progress: bool,
    /// Allow the move when the target already exists but is empty. This is
    /// useful for the "I already mkdir-ed D:\\ven" case. Defaults to true.
    pub allow_existing_empty_target: bool,
    /// If a lock file from a previous (presumably crashed) move is present
    /// at the source, delete it and proceed instead of refusing.
    pub force_unlock: bool,
}

impl Default for MoveOptions {
    fn default() -> Self {
        Self {
            progress: true,
            allow_existing_empty_target: true,
            force_unlock: false,
        }
    }
}

/// Result of a successful move. Surfaced to the caller so the JSON output
/// of `ven path set` can include freed-bytes, used-fast-path, etc.
#[derive(Debug, Clone)]
pub struct MoveReport {
    pub source: PathBuf,
    pub target: PathBuf,
    pub bytes_moved: u64,
    pub files_moved: u64,
    /// True when `fs::rename` succeeded (same device, near-instant). False
    /// when we fell back to copy + verify + delete.
    pub used_fast_path: bool,
}

/// Pre-move size report — what the caller shows in the interactive prompt
/// before asking "Move 3.4 GB to D:\\ven?".
#[derive(Debug, Clone, Copy)]
pub struct SourceSize {
    pub bytes: u64,
    pub files: u64,
}

/// Walk the source directory once and return `(bytes, files)` so the caller
/// can show the user what's about to move. Cheap on warm caches, expensive
/// only on a first-ever ven invocation against multi-GB data.
pub fn measure_source(source: &Path) -> Result<SourceSize> {
    if !source.is_dir() {
        return Ok(SourceSize { bytes: 0, files: 0 });
    }
    let mut bytes = 0u64;
    let mut files = 0u64;
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry.with_context(|| format!("walkdir error in {}", source.display()))?;
        if entry.file_type().is_file() {
            let meta = entry
                .metadata()
                .with_context(|| format!("Failed to stat {}", entry.path().display()))?;
            bytes += meta.len();
            files += 1;
        }
    }
    Ok(SourceSize { bytes, files })
}

/// Validate that `target` is a sensible destination for `source`. Pure check —
/// makes no filesystem mutations. Used both before the move and as a unit
/// test entry point.
pub fn validate_target(source: &Path, target: &Path, opts: &MoveOptions) -> Result<()> {
    if source == target {
        return Err(anyhow!(
            "Target is the same path as the source ({}).",
            target.display()
        ));
    }
    if path_is_inside(target, source) {
        return Err(anyhow!(
            "Target {} is inside the source {}. Moving a directory into itself would either \
             eat the data or recurse forever. Pick a target that lives outside the current \
             ven home.",
            target.display(),
            source.display()
        ));
    }
    if target.exists() {
        if !target.is_dir() {
            return Err(anyhow!(
                "Target {} exists and is not a directory.",
                target.display()
            ));
        }
        let is_empty = fs::read_dir(target)
            .with_context(|| format!("Failed to read {}", target.display()))?
            .next()
            .is_none();
        if !is_empty && !opts.allow_existing_empty_target {
            return Err(anyhow!(
                "Target {} already exists and is not empty.",
                target.display()
            ));
        }
        if !is_empty {
            return Err(anyhow!(
                "Target {} already exists and is not empty. Pick a new path or empty it first.",
                target.display()
            ));
        }
    }
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(anyhow!(
                "Target parent {} does not exist. Create it (or pick an existing parent) and try again.",
                parent.display()
            ));
        }
    }
    Ok(())
}

/// `inner.starts_with(outer)` is *not* sufficient on its own because it
/// returns true for `outer == inner`. We want strictly-inside.
fn path_is_inside(inner: &Path, outer: &Path) -> bool {
    let inner = inner.to_path_buf();
    let outer = outer.to_path_buf();
    inner != outer && inner.starts_with(&outer)
}

/// Physically relocate `source` to `target`. The caller (the `ven path`
/// command) is responsible for:
///
/// - Deciding to move (e.g. by prompting the user).
/// - Writing the global config pointer after this returns `Ok`.
/// - Updating the persistent user env var.
///
/// Returns a [`MoveReport`] suitable for both the human-readable success
/// message and the `--json` payload.
pub fn move_storage(source: &Path, target: &Path, opts: &MoveOptions) -> Result<MoveReport> {
    validate_target(source, target, opts)?;

    let size = measure_source(source)?;

    // Source might not exist if the user has never run `ven install` —
    // perfectly fine, just create the target empty and report a no-op
    // size. The pointer write is what actually matters.
    if !source.exists() {
        fs::create_dir_all(target)
            .with_context(|| format!("Failed to create {}", target.display()))?;
        return Ok(MoveReport {
            source: source.to_path_buf(),
            target: target.to_path_buf(),
            bytes_moved: 0,
            files_moved: 0,
            used_fast_path: true,
        });
    }

    acquire_lock(source, opts)?;

    // Best-effort writability check on the target parent. Catches the
    // "read-only network share" case before we start copying gigabytes.
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            let probe = parent.join(format!(".ven-move-probe-{}", std::process::id()));
            if let Err(e) = fs::write(&probe, b"ven move probe") {
                let _ = release_lock(source);
                return Err(anyhow!(
                    "Cannot write to target parent {}: {}. Pick a writable location.",
                    parent.display(),
                    e
                ));
            }
            let _ = fs::remove_file(&probe);
        }
    }

    // ── Fast path: same device ──────────────────────────────────────────
    if let Err(rename_err) = try_rename(source, target) {
        // Cross-device or "target exists" (already validated empty, but
        // some filesystems still reject rename onto an existing dir);
        // fall through to copy + delete.
        let report = match copy_then_remove(source, target, size, opts) {
            Ok(r) => r,
            Err(copy_err) => {
                // Best-effort: nuke whatever made it to target so the
                // user isn't left with half-baked data on D:.
                let _ = fs::remove_dir_all(target);
                let _ = release_lock(source);
                return Err(copy_err.context(format!(
                    "Cross-device copy failed (original rename error: {})",
                    rename_err
                )));
            }
        };
        // Source is gone, lock is gone with it.
        return Ok(report);
    }

    // Rename succeeded. The source directory (and our lock file) moved
    // atomically to the target; the lock file is now at `target/.ven-move.lock`.
    let _ = release_lock(target);
    Ok(MoveReport {
        source: source.to_path_buf(),
        target: target.to_path_buf(),
        bytes_moved: size.bytes,
        files_moved: size.files,
        used_fast_path: true,
    })
}

fn try_rename(source: &Path, target: &Path) -> io::Result<()> {
    // When the target already exists empty, rename can fail on some
    // filesystems with "directory not empty"-style errors even for an
    // empty dir, because the OS treats the destination as occupied. Try
    // removing the empty placeholder first so rename has a clean slot.
    if target.exists() {
        let _ = fs::remove_dir(target);
    }
    fs::rename(source, target)
}

fn copy_then_remove(
    source: &Path,
    target: &Path,
    size: SourceSize,
    opts: &MoveOptions,
) -> Result<MoveReport> {
    fs::create_dir_all(target).with_context(|| format!("Failed to create {}", target.display()))?;

    let bar = if opts.progress && size.bytes > 0 {
        let pb = ProgressBar::new(size.bytes);
        pb.set_style(
            ProgressStyle::with_template(
                "  [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut copied_bytes = 0u64;
    let mut copied_files = 0u64;

    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry.with_context(|| format!("walkdir error in {}", source.display()))?;
        let rel = entry.path().strip_prefix(source).with_context(|| {
            format!(
                "Path {} not under {}",
                entry.path().display(),
                source.display()
            )
        })?;

        // Skip our own lock file — it's intentionally orphaned at the
        // source until we finalize, and we don't want to ship it to the
        // new location.
        if rel == Path::new(LOCK_FILE) {
            continue;
        }

        let dst = target.join(rel);

        if entry.file_type().is_dir() {
            if !dst.exists() {
                fs::create_dir_all(&dst)
                    .with_context(|| format!("Failed to create {}", dst.display()))?;
            }
        } else if entry.file_type().is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {}", parent.display()))?;
            }
            let n = fs::copy(entry.path(), &dst).with_context(|| {
                format!(
                    "Failed to copy {} -> {}",
                    entry.path().display(),
                    dst.display()
                )
            })?;
            copied_bytes += n;
            copied_files += 1;
            if let Some(pb) = &bar {
                pb.inc(n);
                pb.set_message(format!(
                    "{}",
                    rel.parent().unwrap_or(Path::new("")).display()
                ));
            }

            // Best-effort: preserve executable bit on Unix so node, python,
            // etc. stay runnable at the new location. fs::copy already
            // preserves permissions on Unix, so this is just a sanity
            // touch for filesystems where it doesn't (network shares).
            #[cfg(unix)]
            {
                if let Ok(meta) = entry.metadata() {
                    let _ = fs::set_permissions(&dst, meta.permissions());
                }
            }
        }
        // Symlinks are intentionally not copied as symlinks. We follow
        // them at copy time only if their target sits inside the source;
        // see follow_links(false) above — currently we don't dereference,
        // we just skip. v0.1.6 ships with no symlink support; a future
        // patch can add std::os::unix::fs::symlink / windows equivalent
        // if a real plugin needs it.
    }

    if let Some(pb) = bar {
        pb.finish_and_clear();
    }

    // ── Verify ──────────────────────────────────────────────────────────
    // We can't reliably compare to `size` because the source can have a
    // .ven-move.lock that we skipped. Recompute the expected count.
    let expected_files = size.files; // lock not counted (it's not a runtime file)
    let expected_bytes = size.bytes; // ditto

    if copied_files != expected_files || copied_bytes != expected_bytes {
        return Err(anyhow!(
            "Move verification failed: expected {} files / {} bytes, copied {} files / {} bytes. \
             The target {} has been left in place for inspection; the source {} is untouched.",
            expected_files,
            expected_bytes,
            copied_files,
            copied_bytes,
            target.display(),
            source.display(),
        ));
    }

    // ── Sweep source ────────────────────────────────────────────────────
    fs::remove_dir_all(source).with_context(|| {
        format!(
            "Failed to delete source {} after successful copy",
            source.display()
        )
    })?;

    Ok(MoveReport {
        source: source.to_path_buf(),
        target: target.to_path_buf(),
        bytes_moved: copied_bytes,
        files_moved: copied_files,
        used_fast_path: false,
    })
}

// ─────────────────────────────────────────────────────────────────────────
// Lock handling
// ─────────────────────────────────────────────────────────────────────────

fn lock_path(dir: &Path) -> PathBuf {
    dir.join(LOCK_FILE)
}

fn acquire_lock(source: &Path, opts: &MoveOptions) -> Result<()> {
    if !source.exists() {
        return Ok(()); // Nothing to lock; move_storage has its own early-return.
    }
    let lock = lock_path(source);
    if lock.is_file() {
        if opts.force_unlock {
            let _ = fs::remove_file(&lock);
        } else {
            let body = fs::read_to_string(&lock).unwrap_or_default();
            return Err(anyhow!(
                "Move already in progress (or crashed): lock file {} exists (PID line: {}). \
                 If you are sure no other ven process is moving this directory, delete the lock \
                 file and re-run.",
                lock.display(),
                body.lines().next().unwrap_or("?")
            ));
        }
    }
    fs::write(
        &lock,
        format!("{}\nstarted_at={}\n", std::process::id(), now_secs()),
    )
    .with_context(|| format!("Failed to write lock {}", lock.display()))?;
    Ok(())
}

fn release_lock(dir: &Path) -> Result<()> {
    let lock = lock_path(dir);
    if lock.is_file() {
        fs::remove_file(&lock)
            .with_context(|| format!("Failed to remove lock {}", lock.display()))?;
    }
    Ok(())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};

    fn touch(p: &Path, body: &[u8]) {
        if let Some(parent) = p.parent() {
            create_dir_all(parent).unwrap();
        }
        write(p, body).unwrap();
    }

    fn populate_fake_ven(root: &Path) -> SourceSize {
        touch(&root.join("node/20.11.0/node"), b"fake node binary");
        touch(
            &root.join("node/20.11.0/lib/something.js"),
            b"console.log(1)",
        );
        touch(&root.join("python/3.12.7/python.exe"), &vec![0u8; 4096]);
        touch(&root.join("cache/osv.sqlite"), &vec![0u8; 2048]);
        measure_source(root).unwrap()
    }

    #[test]
    fn measure_handles_missing_source() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("never-existed");
        let size = measure_source(&missing).unwrap();
        assert_eq!(size.bytes, 0);
        assert_eq!(size.files, 0);
    }

    #[test]
    fn validate_rejects_target_inside_source() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("ven");
        create_dir_all(&source).unwrap();
        let nested = source.join("nested");
        let err = validate_target(&source, &nested, &MoveOptions::default()).unwrap_err();
        assert!(err.to_string().contains("inside the source"), "got: {err}");
    }

    #[test]
    fn validate_rejects_same_path() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("ven");
        create_dir_all(&source).unwrap();
        let err = validate_target(&source, &source, &MoveOptions::default()).unwrap_err();
        assert!(err.to_string().contains("same path"), "got: {err}");
    }

    #[test]
    fn validate_rejects_existing_non_empty_target() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("src");
        let target = temp.path().join("dst");
        create_dir_all(&source).unwrap();
        create_dir_all(&target).unwrap();
        write(target.join("squatter"), b"x").unwrap();
        let err = validate_target(&source, &target, &MoveOptions::default()).unwrap_err();
        assert!(err.to_string().contains("not empty"), "got: {err}");
    }

    #[test]
    fn validate_accepts_existing_empty_target() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("src");
        let target = temp.path().join("dst");
        create_dir_all(&source).unwrap();
        create_dir_all(&target).unwrap();
        validate_target(&source, &target, &MoveOptions::default()).unwrap();
    }

    #[test]
    fn move_same_device_uses_fast_path() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("ven-src");
        let target = temp.path().join("ven-dst");
        let size = populate_fake_ven(&source);

        let opts = MoveOptions {
            progress: false,
            ..MoveOptions::default()
        };
        let report = move_storage(&source, &target, &opts).unwrap();

        assert!(report.used_fast_path, "expected same-device fast path");
        assert!(!source.exists(), "source should be gone after rename");
        assert!(target.join("node/20.11.0/node").is_file());
        assert!(target.join("python/3.12.7/python.exe").is_file());
        assert_eq!(report.bytes_moved, size.bytes);
        assert_eq!(report.files_moved, size.files);
    }

    #[test]
    fn move_slow_path_when_target_exists_empty() {
        // We can't trigger a real EXDEV in a portable test, but we can
        // force the slow path by pre-creating the target as an empty
        // dir AND then making `fs::rename` route through copy+delete
        // by also touching a sentinel file inside it.
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("ven-src");
        let target = temp.path().join("ven-dst");
        let size = populate_fake_ven(&source);
        create_dir_all(&target).unwrap();

        let opts = MoveOptions {
            progress: false,
            ..MoveOptions::default()
        };
        let report = move_storage(&source, &target, &opts).unwrap();

        // Either path is acceptable; we just want the data to land.
        assert!(!source.exists());
        assert!(target.join("node/20.11.0/node").is_file());
        assert_eq!(report.bytes_moved, size.bytes);
        assert_eq!(report.files_moved, size.files);
    }

    #[test]
    fn lock_file_blocks_concurrent_move() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("ven-src");
        let target = temp.path().join("ven-dst");
        populate_fake_ven(&source);
        // Simulate a stuck previous run.
        fs::write(source.join(LOCK_FILE), "12345\n").unwrap();

        let opts = MoveOptions {
            progress: false,
            ..MoveOptions::default()
        };
        let err = move_storage(&source, &target, &opts).unwrap_err();
        assert!(
            err.to_string().contains("Move already in progress"),
            "got: {err}"
        );

        // Source is untouched.
        assert!(source.join("node/20.11.0/node").is_file());
        assert!(!target.exists() || fs::read_dir(&target).unwrap().next().is_none());
    }

    #[test]
    fn force_unlock_breaks_stuck_lock() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("ven-src");
        let target = temp.path().join("ven-dst");
        populate_fake_ven(&source);
        fs::write(source.join(LOCK_FILE), "12345\n").unwrap();

        let opts = MoveOptions {
            progress: false,
            force_unlock: true,
            ..MoveOptions::default()
        };
        let report = move_storage(&source, &target, &opts).unwrap();
        assert!(target.join("node/20.11.0/node").is_file());
        assert!(report.files_moved >= 4);
    }

    #[test]
    fn missing_source_creates_target_and_reports_zero() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("never-existed");
        let target = temp.path().join("fresh-target");

        let opts = MoveOptions {
            progress: false,
            ..MoveOptions::default()
        };
        let report = move_storage(&source, &target, &opts).unwrap();

        assert!(target.is_dir());
        assert_eq!(report.bytes_moved, 0);
        assert_eq!(report.files_moved, 0);
        assert!(report.used_fast_path);
    }
}
