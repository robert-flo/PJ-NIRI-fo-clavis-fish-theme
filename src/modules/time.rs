use crate::color::Color;
use crate::config::ColorPair;
use crate::segment::{capsule, styled, Segment};

fn content() -> String {
    let current = jiff::Zoned::now();
    format!(" {}", current.strftime("%H:%M:%S"))
}

pub fn render(colors: ColorPair) -> Segment {
    let content = content();
    capsule(colors, &content)
}

pub fn render_plain(foreground: Color) -> Segment {
    let content = content();
    styled(foreground, &content)
}

#[cfg(test)]
mod tests {
    use super::{render, render_plain};
    use crate::config::Config;

    #[test]
    fn normal_time_is_a_colored_capsule() {
        let config = Config::default();
        let segment = render(config.colors.time);
        assert!(segment.rendered.contains(""));
        assert!(segment.rendered.contains(""));
        assert!(segment.rendered.contains("48;2;64;71;83m"));
        assert!(segment.rendered.contains(" "));
        assert_eq!(segment.visible_width, 12);
    }

    #[test]
    fn transient_time_is_foreground_only_text() {
        let config = Config::default();
        let segment = render_plain(config.colors.time.foreground);
        assert!(!segment.rendered.contains(""));
        assert!(!segment.rendered.contains(""));
        assert!(!segment.rendered.contains("48;2;64;71;83m"));
        assert!(segment.rendered.contains("38;2;220;227;242m"));
        assert!(segment.rendered.contains(" "));
        assert_eq!(segment.visible_width, 10);
    }
}
