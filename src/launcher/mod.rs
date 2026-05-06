//! Terminal launcher scaffolding (phase 2: shell detection only).

pub mod shell;

pub use shell::{detect_shell, ShellKind};
