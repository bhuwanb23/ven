# Phase 1: Security-Critical Path Review — Detailed Todos

## Todo 1: Fix Zip Slip Vulnerability in extract.rs
**File:** `src/core/extract.rs:18`
**Severity:** CRITICAL
**Issue:** `entry.mangled_name()` does NOT sanitize path traversal. A malicious zip with entries like `../../etc/passwd` escapes the destination directory.
**Fix:** Validate that the resolved output path is within the destination directory. Reject entries with `..` components.
**Test:** Add unit test with a crafted zip containing path traversal entries.

## Todo 2: Fix Tar Path Traversal in extract.rs
**File:** `src/core/extract.rs:47`
**Severity:** CRITICAL
**Issue:** `archive.unpack(dest)` does NOT protect against `../` in tar entry names by default.
**Fix:** Use `archive.entries()` iterator, validate each entry's path before unpacking, reject entries that escape `dest`.
**Test:** Add unit test with a crafted tar containing path traversal entries.

## Todo 3: Make SHA-256 Comparison Timing-Safe in integrity.rs
**File:** `src/core/integrity.rs:36`
**Severity:** IMPORTANT
**Issue:** `actual == expected` string comparison is NOT constant-time. Vulnerable to timing side-channel attacks.
**Fix:** Use `subtle::ConstantTimeEq` or manual byte-by-byte comparison with constant-time OR.
**Test:** Verify the comparison still works correctly.

## Todo 4: Fail on Unavailable Checksum in download.rs
**File:** `src/core/download.rs:156`
**Severity:** IMPORTANT
**Issue:** When checksum fetch fails, `print_checksum_unavailable` prints a warning but continues. This allows unverified downloads.
**Fix:** Make checksum verification mandatory — return error instead of warning when checksum is unavailable.
**Test:** Verify error is returned when checksum fetch fails.

## Todo 5: Escape ven_path in Shell Hook Generation
**File:** `src/shell/mod.rs:60-62, 231-233`
**Severity:** IMPORTANT
**Issue:** `ven_path` from `std::env::current_exe()` is embedded directly in shell code without escaping. Paths with spaces/quotes break or are exploitable.
**Fix:** Shell-escape the path for each target shell (single-quote for bash/zsh/fish, double-quote with escape for PowerShell).
**Test:** Test hook generation with paths containing spaces and special characters.

## Todo 6: Make Lockfile Write Atomic in ven_lock.rs
**File:** `src/intelligence/ven_lock.rs:179-183`
**Severity:** IMPORTANT
**Issue:** `write_path` uses `fs::write` which is not atomic. Crash mid-write corrupts the lockfile.
**Fix:** Use temp file + rename pattern (like `ven_config.rs` already does).
**Test:** Verify atomic write behavior.

## Todo 7: Add Shell Profile Backup Before Modification
**File:** `src/bin/setup/install_steps.rs` and platform-specific files
**Severity:** IMPORTANT
**Issue:** No backup created before modifying rc files. Mid-write failure corrupts the profile.
**Fix:** Create `.bak` backup before modification, restore on failure.
**Test:** Test backup creation and restoration.

## Todo 8: Add Decompression Bomb Protection in extract.rs
**File:** `src/core/extract.rs`
**Severity:** MINOR
**Issue:** No size limits on extracted content. A decompression bomb could exhaust disk.
**Fix:** Track total extracted bytes, abort if exceeding a reasonable limit (e.g., 2GB for runtime archives).
**Test:** Test with an archive that exceeds the limit.

## Todo 9: Validate Symlink Targets in Archive Extraction
**File:** `src/core/extract.rs`
**Severity:** MINOR
**Issue:** Symlinks in archives could point outside the extraction directory.
**Fix:** For tar extraction, skip or reject symlinks that point outside `dest`. For zip, check symlink targets.
**Test:** Test extraction with symlinks pointing outside the destination.
