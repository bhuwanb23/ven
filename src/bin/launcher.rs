//! Entry point for `ven-launcher` — terminal spawner.

use ven::launcher::detect_shell;

fn main() {
    println!("Detected shell: {}", detect_shell());
}
