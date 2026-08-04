//! ANSI-aware terminal cell width helpers.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

/// Return the number of terminal cells occupied by a string.
///
/// Prompt ANSI CSI/OSC sequences are skipped before Unicode graphemes are
/// measured. A grapheme uses the widest code point in it, matching the width
/// strategy used by Starship for combining marks and emoji sequences.
pub fn visible_width(text: &str) -> usize {
    let plain = strip_ansi(text);
    UnicodeSegmentation::graphemes(plain.as_str(), true)
        .map(|grapheme| {
            grapheme
                .chars()
                .filter_map(UnicodeWidthChar::width)
                .max()
                .unwrap_or(0)
        })
        .sum()
}

fn strip_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b {
            if let Some(next) = ansi_sequence_end(bytes, index) {
                index = next;
                continue;
            }
        }
        let character = text[index..].chars().next().unwrap_or('\u{fffd}');
        result.push(character);
        index += character.len_utf8();
    }
    result
}

fn ansi_sequence_end(bytes: &[u8], start: usize) -> Option<usize> {
    let next = *bytes.get(start + 1)?;
    match next {
        b'[' => {
            let mut index = start + 2;
            while let Some(byte) = bytes.get(index).copied() {
                index += 1;
                if (0x40..=0x7e).contains(&byte) {
                    return Some(index);
                }
            }
            Some(bytes.len())
        }
        b']' => {
            let mut index = start + 2;
            while let Some(byte) = bytes.get(index).copied() {
                index += 1;
                if byte == 0x07 {
                    return Some(index);
                }
                if byte == 0x1b && bytes.get(index).copied() == Some(b'\\') {
                    return Some(index + 1);
                }
            }
            Some(bytes.len())
        }
        _ => None,
    }
}

/// Truncate at grapheme boundaries without splitting UTF-8 or ANSI sequences.
/// This helper is used only with plain module text, but it also handles ANSI
/// input for tests and future callers.
pub fn truncate_visible(text: &str, max_width: usize) -> String {
    if visible_width(text) <= max_width {
        return text.to_owned();
    }

    let plain = strip_ansi(text);
    let mut result = String::new();
    let mut width = 0;
    for grapheme in UnicodeSegmentation::graphemes(plain.as_str(), true) {
        let grapheme_width = grapheme
            .chars()
            .filter_map(UnicodeWidthChar::width)
            .max()
            .unwrap_or(0);
        if width + grapheme_width > max_width {
            break;
        }
        result.push_str(grapheme);
        width += grapheme_width;
    }
    result
}

/// Truncate plain text at a grapheme boundary and reserve one cell for an
/// ellipsis when truncation occurs.
pub fn truncate_visible_with_ellipsis(text: &str, max_width: usize) -> String {
    if visible_width(text) <= max_width {
        return text.to_owned();
    }
    if max_width == 0 {
        return String::new();
    }
    let mut result = truncate_visible(text, max_width.saturating_sub(1));
    result.push('…');
    result
}

#[cfg(test)]
mod tests {
    use super::{truncate_visible, truncate_visible_with_ellipsis, visible_width};

    #[test]
    fn measures_ansi_and_unicode_cells() {
        assert_eq!(visible_width("abc"), 3);
        assert_eq!(visible_width("\x1b[38;2;1;2;3m中文\x1b[0m"), 4);
        assert_eq!(visible_width(""), 1);
        assert_eq!(visible_width("󰘬"), 1);
        assert_eq!(visible_width("😀"), 2);
        assert_eq!(visible_width("e\u{301}"), 1);
    }

    #[test]
    fn truncates_on_grapheme_boundaries() {
        assert_eq!(truncate_visible("中文路径", 5), "中文");
        assert_eq!(truncate_visible("e\u{301}x", 1), "e\u{301}");
        assert_eq!(truncate_visible("abcdef", 6), "abcdef");
        assert_eq!(truncate_visible_with_ellipsis("abcdef", 4), "abc…");
        assert_eq!(truncate_visible_with_ellipsis("中文路径", 4), "中…");
    }
}
