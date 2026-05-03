use std::path::PathBuf;
use ven::core::{find_ven_toml, parse_ven_toml};

#[test]
fn test_example_directory_config() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("example");
    
    // Find ven.toml from the root example dir
    let toml_path = find_ven_toml(&d).expect("Should find ven.toml in example dir");
    assert!(toml_path.ends_with("example\\ven.toml") || toml_path.ends_with("example/ven.toml"));
    
    // Parse the actual example file
    let config = parse_ven_toml(&toml_path).expect("Should parse example ven.toml");
    
    assert_eq!(config.runtime.node, "25.9.0");
    assert!(config.packages.is_empty());
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
