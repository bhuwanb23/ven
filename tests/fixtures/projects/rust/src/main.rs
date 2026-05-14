//! Fixture: mixes declared, stdlib, and ghost crate usage.
//!   serde   -> declared in Cargo.toml (not a ghost)
//!   std     -> stdlib (ignored by scanner)
//!   anyhow  -> ghost
//!   tokio   -> ghost

use std::collections::HashMap;

use serde::Serialize;
use anyhow::Result;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct Sample {
    name: String,
}

#[allow(dead_code)]
fn main() -> Result<()> {
    let _map: HashMap<String, Sample> = HashMap::new();
    let _lock: Option<Mutex<()>> = None;
    Ok(())
}
