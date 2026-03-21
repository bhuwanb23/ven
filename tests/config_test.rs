use std::path::PathBuf;
use ven::core::config::{find_ven_toml, parse_ven_toml, version_spec_resolver};

#[test]
fn test_example_directory_config() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("example");
    
    // Find ven.toml from the root example dir
    let toml_path = find_ven_toml(&d).expect("Should find ven.toml in example dir");
    assert!(toml_path.ends_with("example\\ven.toml") || toml_path.ends_with("example/ven.toml"));
    
    // Parse the actual example file
    let config = parse_ven_toml(&toml_path).expect("Should parse example ven.toml");
    
    assert_eq!(config.runtime.node, "20.11.1");
    assert_eq!(config.packages.get("express").unwrap(), "^4.18.2");
    assert_eq!(config.env.get("NODE_ENV").unwrap(), "development");
}

#[test]
fn test_nested_example_directory() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("example");
    d.push("a");
    d.push("b");
    d.push("c");
    
    // Walk up from example/a/b/c to find ven.toml in example/
    let toml_path = find_ven_toml(&d).expect("Should walk up and find ven.toml");
    assert!(toml_path.ends_with("example\\ven.toml") || toml_path.ends_with("example/ven.toml"));
}

#[test]
fn test_version_resolver() {
    assert_eq!(version_spec_resolver("latest"), "latest");
    assert_eq!(version_spec_resolver("18"), "18");
    assert_eq!(version_spec_resolver(">=20"), ">=20");
}
