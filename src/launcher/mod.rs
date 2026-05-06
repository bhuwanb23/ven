//! Terminal launcher: shell detection, `ven.toml`-derived env preview, and (later) spawning.

pub mod env;
pub mod greeting;
pub mod paths;
pub mod quote;
pub mod shell;
pub mod spawn;

pub use shell::{detect_shell, ShellKind};
