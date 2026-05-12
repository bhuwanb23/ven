//! Integration tests for the ghost dependency scanner using fixture
//! projects under `tests/fixtures/projects/<lang>/`. Cross-platform: only
//! depends on file walking + regex; same expected output on Windows /
//! macOS / Linux.

use std::path::PathBuf;

use ven::core::config::VenConfig;
use ven::core::ghost_scanner::{scan_project, GhostReport};
use ven::intelligence::graph::RuntimeKind;

fn fixture(lang: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push("fixtures");
    d.push("projects");
    d.push(lang);
    d
}

fn names(report: &GhostReport) -> Vec<String> {
    report.ghosts.iter().map(|g| g.name.clone()).collect()
}

#[test]
fn node_fixture_finds_axios_zlib_lodash() {
    let root = fixture("node");
    let cfg = VenConfig::default();
    let report = scan_project(&root, &cfg, RuntimeKind::NpmFamily).unwrap();
    let n = names(&report);

    // Declared via package.json — should NOT show up.
    assert!(!n.contains(&"express".to_string()), "express should be declared, got {:?}", n);
    // Stdlib — should NOT show up.
    assert!(!n.contains(&"fs".to_string()));
    // Real ghosts.
    assert!(n.contains(&"axios".to_string()), "axios missing in {:?}", n);
    assert!(
        n.iter().any(|s| s == "@scope/zlib-tools"),
        "scoped ghost missing in {:?}",
        n
    );
    assert!(n.contains(&"lodash".to_string()), "lodash dynamic import missing in {:?}", n);
}

#[test]
fn python_fixture_finds_flask_pyyaml_sklearn() {
    let root = fixture("python");
    let cfg = VenConfig::default();
    let report = scan_project(&root, &cfg, RuntimeKind::Python).unwrap();
    let n = names(&report);

    assert!(!n.contains(&"requests".to_string()), "requests declared, got {:?}", n);
    assert!(!n.iter().any(|s| s == "os" || s == "sys"), "stdlib leaked: {:?}", n);
    assert!(n.contains(&"flask".to_string()), "flask missing in {:?}", n);
    // Rename table: yaml → PyYAML (canonicalised lowercase).
    assert!(n.contains(&"pyyaml".to_string()), "pyyaml (rename) missing in {:?}", n);
    assert!(n.contains(&"scikit-learn".to_string()), "scikit-learn (rename) missing in {:?}", n);
}

#[test]
fn rust_fixture_finds_anyhow_and_tokio() {
    let root = fixture("rust");
    let cfg = VenConfig::default();
    let report = scan_project(&root, &cfg, RuntimeKind::Rust).unwrap();
    let n = names(&report);

    assert!(!n.contains(&"serde".to_string()));
    assert!(!n.iter().any(|s| s == "std" || s == "core" || s == "alloc"));
    assert!(n.contains(&"anyhow".to_string()), "anyhow missing in {:?}", n);
    assert!(n.contains(&"tokio".to_string()), "tokio missing in {:?}", n);
}

#[test]
fn go_fixture_finds_logrus() {
    let root = fixture("go");
    let cfg = VenConfig::default();
    let report = scan_project(&root, &cfg, RuntimeKind::Go).unwrap();
    let n = names(&report);

    // Declared.
    assert!(
        !n.iter().any(|s| s.contains("gin-gonic")),
        "gin should be declared, got {:?}",
        n
    );
    // Stdlib (no dot in first segment) — should NOT show up.
    assert!(!n.iter().any(|s| s == "fmt" || s == "net/http"));
    // Real ghost.
    assert!(
        n.iter().any(|s| s == "github.com/sirupsen/logrus"),
        "logrus missing in {:?}",
        n
    );
}
