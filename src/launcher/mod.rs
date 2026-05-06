//! Terminal launcher: shell detection, `ven.toml`-derived env preview, and (later) spawning.

pub mod env;
pub mod shell;

pub use shell::{detect_shell, ShellKind};
