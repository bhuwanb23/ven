use std::path::{Path, PathBuf};
use ven::core::{find_ven_toml, parse_ven_toml};

/// Path to the directory containing the canonical test `ven.toml`.
///
/// The fixture lives under `tests/fixtures/config/` (a tracked path)
/// rather than `example/` because `/example/` is gitignored as a
/// user-facing scratchpad — see `.gitignore`. Anchoring on
/// `CARGO_MANIFEST_DIR` keeps the lookup independent of CWD.
fn fixture_root() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push("fixtures");
    d.push("config");
    d
}

/// Cross-platform end-of-path check that doesn't care about
/// `\` vs `/` separators (which differ between Windows and Unix CI runners).
fn ends_with_components(path: &Path, expected_tail: &[&str]) -> bool {
    let comps: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if comps.len() < expected_tail.len() {
        return false;
    }
    let tail = &comps[comps.len() - expected_tail.len()..];
    tail.iter()
        .zip(expected_tail.iter())
        .all(|(a, b)| a == b)
}

#[test]
fn test_example_directory_config() {
    let d = fixture_root();

    let toml_path =
        find_ven_toml(&d).expect("Should find ven.toml in tests/fixtures/config");
    assert!(
        ends_with_components(&toml_path, &["config", "ven.toml"]),
        "expected ../config/ven.toml, got {}",
        toml_path.display()
    );

    let config = parse_ven_toml(&toml_path).expect("Should parse fixture ven.toml");

    assert_eq!(config.runtime.node, "25.9.0");
    assert!(config.packages.is_empty());
    assert!(
        config.venv.auto_path,
        "omit [venv] => default auto_path=true"
    );
}

#[test]
fn test_nested_example_directory() {
    let mut d = fixture_root();
    d.push("a");
    d.push("b");
    d.push("c");

    // Walk up from .../config/a/b/c to find ven.toml in .../config/.
    let toml_path = find_ven_toml(&d).expect("Should walk up and find ven.toml");
    assert!(
        ends_with_components(&toml_path, &["config", "ven.toml"]),
        "expected ../config/ven.toml, got {}",
        toml_path.display()
    );
}
