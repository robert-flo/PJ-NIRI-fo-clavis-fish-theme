//! Small, strict true-colour support used by the prompt renderer.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Parse exactly `#RRGGBB`. Short forms and named colours are deliberately
    /// not accepted so a malformed Matugen/user value can safely fall back.
    pub fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 7 || bytes[0] != b'#' {
            return None;
        }

        fn hex(byte: u8) -> Option<u8> {
            match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                b'A'..=b'F' => Some(byte - b'A' + 10),
                _ => None,
            }
        }

        let pair = |high: u8, low: u8| Some(hex(high)? * 16 + hex(low)?);
        Some(Self::rgb(
            pair(bytes[1], bytes[2])?,
            pair(bytes[3], bytes[4])?,
            pair(bytes[5], bytes[6])?,
        ))
    }

    pub fn foreground_escape(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.red, self.green, self.blue)
    }

    pub fn background_escape(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.red, self.green, self.blue)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn parses_only_full_hex_colors() {
        assert_eq!(Color::parse("#c0C7d5"), Some(Color::rgb(192, 199, 213)));
        assert_eq!(Color::parse("#fff"), None);
        assert_eq!(Color::parse("c0c7d5"), None);
        assert_eq!(Color::parse("#gggggg"), None);
    }
}
