use std::env;
use std::io;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

const BINARY_PLACEHOLDER: &str = "__CLAVIS_FISH_THEME_BINARY__";

pub fn print_fish() -> io::Result<()> {
    let executable = env::current_exe()?;
    #[cfg(unix)]
    let path_bytes = executable.as_os_str().as_bytes();
    #[cfg(not(unix))]
    let path_bytes = executable.to_string_lossy().as_bytes();
    let encoded = fish_variable_escape(path_bytes);
    let script = include_str!("init/clavis.fish").replace(BINARY_PLACEHOLDER, &encoded);
    print!("{script}");
    Ok(())
}

/// Encode a path using Fish's `string escape --style=var` convention. The
/// generated script decodes it with the Fish builtin, so spaces, quotes,
/// dollar signs, parentheses, and non-UTF-8 bytes cannot become code.
fn fish_variable_escape(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        if byte.is_ascii_alphanumeric() {
            result.push(*byte as char);
        } else if *byte == b'_' {
            result.push_str("__");
        } else {
            result.push('_');
            result.push_str(&format!("{byte:02X}"));
            result.push('_');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::fish_variable_escape;

    #[test]
    fn uses_fish_variable_escaping_for_paths() {
        assert_eq!(fish_variable_escape(b"/tmp/a b_1"), "_2F_tmp_2F_a_20_b__1");
        assert!(!fish_variable_escape(b"a'b$(c)").contains('\''));
    }
}
