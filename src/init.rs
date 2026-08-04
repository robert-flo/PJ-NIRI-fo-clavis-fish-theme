use std::env;
use std::io;

const BINARY_PLACEHOLDER: &str = "__CLAVIS_FISH_THEME_BINARY__";

pub fn print_fish() -> io::Result<()> {
    let executable = env::current_exe()?;
    let executable = executable.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "executable path is not valid UTF-8 and cannot be embedded in Fish source",
        )
    })?;
    let quoted_path = fish_quote(executable);
    let script = include_str!("init/clavis.fish").replace(BINARY_PLACEHOLDER, &quoted_path);
    print!("{script}");
    Ok(())
}

/// Quote a path as a literal Fish string.
///
/// This is deliberately string quoting rather than Fish's `var` escaping.
/// The latter is intended for encoding variable names and turns characters
/// such as `.` and `/` into `_2E` and `_2F`. A single-quoted literal keeps all
/// path characters literal; an apostrophe is represented by closing the
/// quote, escaping the apostrophe, and opening it again.
fn fish_quote(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('\'');
    for character in value.chars() {
        if character == '\'' {
            result.push_str("'\\''");
        } else {
            result.push(character);
        }
    }
    result.push('\'');
    result
}

#[cfg(test)]
mod tests {
    use super::fish_quote;

    #[test]
    fn keeps_path_punctuation_literal() {
        let quoted = fish_quote("/home/user/.cargo/bin/clavis-fish-theme");
        assert_eq!(quoted, "'/home/user/.cargo/bin/clavis-fish-theme'");
        assert!(!quoted.contains("_2E"));
        assert!(!quoted.contains("_2F"));
        assert_eq!(
            fish_quote("/home/user/.local/bin/clavis-fish-theme"),
            "'/home/user/.local/bin/clavis-fish-theme'"
        );
    }

    #[test]
    fn quotes_shell_special_characters_without_encoding_them() {
        assert_eq!(
            fish_quote("/tmp/My Tools/bin/clavis-fish-theme"),
            "'/tmp/My Tools/bin/clavis-fish-theme'"
        );
        assert_eq!(
            fish_quote("/tmp/it's-tools/bin/clavis-fish-theme"),
            "'/tmp/it'\\''s-tools/bin/clavis-fish-theme'"
        );
        assert_eq!(
            fish_quote("/tmp/$tools/$(not-a-command)/`literal`"),
            "'/tmp/$tools/$(not-a-command)/`literal`'"
        );
    }
}
