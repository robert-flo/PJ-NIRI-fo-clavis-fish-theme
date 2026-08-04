use crate::config::ColorPair;
use crate::prompt::PromptState;
use crate::segment::capsule;

pub fn render(colors: ColorPair, state: &PromptState) -> Option<crate::segment::Segment> {
    if !state.command_seen {
        return None;
    }
    Some(capsule(
        colors,
        &format!("󰔚 {}", format_duration(state.duration_ms.unwrap_or(0))),
    ))
}

/// Format Fish's millisecond `$CMD_DURATION` without a minimum display
/// threshold. Rounding is kept below the next unit boundary (59.999s is
/// still rendered as 59.9s).
pub fn format_duration(milliseconds: u64) -> String {
    if milliseconds < 1_000 {
        return format!("{milliseconds}ms");
    }

    if milliseconds < 60_000 {
        let tenths = ((milliseconds + 50) / 100).min(599);
        let seconds = tenths / 10;
        let decimal = tenths % 10;
        return if decimal == 0 {
            format!("{seconds}s")
        } else {
            format!("{seconds}.{decimal}s")
        };
    }

    let total_seconds = milliseconds / 1_000;
    let minutes = total_seconds / 60;
    if minutes < 60 {
        return format!("{minutes}m {:02}s", total_seconds % 60);
    }

    let hours = minutes / 60;
    format!("{hours}h {:02}m", minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::format_duration;

    #[test]
    fn formats_milliseconds_and_seconds() {
        assert_eq!(format_duration(0), "0ms");
        assert_eq!(format_duration(186), "186ms");
        assert_eq!(format_duration(999), "999ms");
        assert_eq!(format_duration(1_000), "1s");
        assert_eq!(format_duration(1_862), "1.9s");
        assert_eq!(format_duration(59_999), "59.9s");
    }

    #[test]
    fn formats_minutes_and_hours() {
        assert_eq!(format_duration(60_000), "1m 00s");
        assert_eq!(format_duration(134_000), "2m 14s");
        assert_eq!(format_duration(3_599_000), "59m 59s");
        assert_eq!(format_duration(3_600_000), "1h 00m");
        assert_eq!(format_duration(5_160_000), "1h 26m");
    }
}
