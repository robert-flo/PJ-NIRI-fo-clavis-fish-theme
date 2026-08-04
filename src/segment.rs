use crate::color::Color;
use crate::config::ColorPair;
use crate::width::visible_width;

pub const RESET: &str = "\x1b[0m";
pub const LEFT_CAP: &str = "";
pub const RIGHT_CAP: &str = "";
pub const CONNECTOR_ICON: &str = " 󰜥 ";
pub const PROMPT_START: &str = "╭─ ";
pub const PROMPT_END: &str = "╰─ ";
pub const ARROW_ICON: &str = "";
pub const TRANSIENT_ARROW_ICON: &str = "";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub rendered: String,
    pub visible_width: usize,
}

pub fn styled(color: Color, text: &str) -> Segment {
    Segment {
        rendered: format!("{}{}{}", color.foreground_escape(), text, RESET),
        visible_width: visible_width(text),
    }
}

/// Render a capsule without implicit left/right padding. Any space in
/// `content` is intentional module content (for example icon + branch name).
pub fn capsule(colors: ColorPair, content: &str) -> Segment {
    let rendered = format!(
        "{}{}{}{}{}{}{}{}{}",
        colors.background.foreground_escape(),
        LEFT_CAP,
        colors.background.background_escape(),
        colors.foreground.foreground_escape(),
        content,
        RESET,
        colors.background.foreground_escape(),
        RIGHT_CAP,
        RESET,
    );
    Segment {
        rendered,
        visible_width: visible_width(LEFT_CAP) + visible_width(content) + visible_width(RIGHT_CAP),
    }
}

pub fn connector(color: Color) -> Segment {
    styled(color, CONNECTOR_ICON)
}

pub fn arrow(color: Color) -> Segment {
    styled(color, &format!("{ARROW_ICON} "))
}

pub fn transient_arrow(color: Color) -> Segment {
    styled(color, &format!("{TRANSIENT_ARROW_ICON} "))
}

pub fn prompt_start(color: Color) -> Segment {
    styled(color, PROMPT_START)
}

pub fn prompt_end(color: Color) -> Segment {
    styled(color, PROMPT_END)
}

#[cfg(test)]
mod tests {
    use super::{capsule, connector, ColorPair, CONNECTOR_ICON};
    use crate::color::Color;

    #[test]
    fn capsule_has_no_implicit_padding() {
        let segment = capsule(
            ColorPair {
                background: Color::rgb(1, 2, 3),
                foreground: Color::rgb(4, 5, 6),
            },
            "󰔚 186ms",
        );
        assert_eq!(segment.visible_width, 1 + 7 + 1);
        assert!(!segment.rendered.contains(" "));
        assert!(!segment.rendered.contains(" "));
    }

    #[test]
    fn wave_connector_has_one_cell_of_space_on_each_side() {
        let segment = connector(Color::rgb(1, 2, 3));
        assert_eq!(CONNECTOR_ICON, " 󰜥 ");
        assert_eq!(segment.visible_width, 3);
        assert!(segment.rendered.contains(CONNECTOR_ICON));
    }
}
