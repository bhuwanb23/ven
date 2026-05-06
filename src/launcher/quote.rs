//! POSIX shell single-quoting for init snippets.

/// Bash/zsh-safe single-quoted literal (for `printf` / `echo` arguments).
pub fn bash_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}
