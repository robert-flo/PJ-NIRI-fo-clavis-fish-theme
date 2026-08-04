use crate::config::Config;
use crate::modules::RenderedModules;
use crate::prompt::PromptState;
use crate::segment::{arrow, connector, Segment};

struct Joined {
    rendered: String,
    width: usize,
}

#[derive(Clone, Copy)]
struct Selection {
    show_duration: bool,
    show_path: bool,
    path_variant: usize,
    show_git: bool,
    git_variant: usize,
    show_language: bool,
    show_time: bool,
}

pub fn render_full(config: &Config, state: &PromptState, columns: usize) -> String {
    let modules = crate::modules::render(config, state);
    let connector = connector(config.colors.connector);
    let mut selection = Selection {
        show_duration: modules.duration.is_some(),
        show_path: !modules.path.is_empty(),
        path_variant: 0,
        show_git: !modules.git.is_empty(),
        git_variant: 0,
        show_language: modules.language.is_some(),
        show_time: modules.time.is_some(),
    };

    loop {
        if fits(config, columns, &modules, &selection, &connector) {
            break;
        }

        // The order is a user-visible contract for narrow terminals.
        if selection.show_language {
            selection.show_language = false;
        } else if selection.show_time {
            selection.show_time = false;
        } else if selection.show_git && selection.git_variant + 1 < modules.git.len() {
            selection.git_variant += 1;
        } else if selection.show_git {
            selection.show_git = false;
        } else if selection.show_path && selection.path_variant + 1 < modules.path.len() {
            selection.path_variant += 1;
        } else if selection.show_path {
            selection.show_path = false;
        } else if selection.show_duration {
            selection.show_duration = false;
        } else {
            break;
        }
    }

    let (left, right) = joined_sides(&modules, &selection, &connector);
    let first_line = first_line(config, columns, &left, &right);
    let arrow_color = if state.failed() {
        // A failed command changes only the arrow foreground. It does not
        // recolour duration, Git, or time capsules.
        config.colors.error.foreground
    } else {
        config.colors.arrow
    };
    let arrow_segment = arrow(arrow_color);
    let left_margin = effective_margins(config, columns).0;
    format!(
        "{first_line}\n{}{}",
        " ".repeat(left_margin),
        arrow_segment.rendered
    )
}

pub fn render_transient_left(config: &Config, state: &PromptState, columns: usize) -> String {
    let (left_margin, _) = effective_margins(config, columns);
    let color = if state.failed() {
        config.colors.error.foreground
    } else {
        config.colors.arrow
    };
    let arrow_segment = arrow(color);
    format!("{}{}", " ".repeat(left_margin), arrow_segment.rendered)
}

pub fn render_transient_right(config: &Config, columns: usize) -> String {
    if !config.modules.time {
        return String::new();
    }
    let time = crate::modules::time::render(config.colors.time);
    let right_margin = effective_margins(config, columns).1;
    format!("{}{}", time.rendered, " ".repeat(right_margin))
}

fn joined_sides(
    modules: &RenderedModules,
    selection: &Selection,
    connector: &Segment,
) -> (Joined, Joined) {
    let mut left = Vec::new();
    if selection.show_duration {
        if let Some(segment) = modules.duration.as_ref() {
            left.push(segment);
        }
    }
    if selection.show_path {
        if let Some(segment) = modules.path.get(selection.path_variant) {
            left.push(segment);
        }
    }
    if selection.show_git {
        if let Some(segment) = modules.git.get(selection.git_variant) {
            left.push(segment);
        }
    }

    let mut right = Vec::new();
    if selection.show_language {
        if let Some(segment) = modules.language.as_ref() {
            right.push(segment);
        }
    }
    if selection.show_time {
        if let Some(segment) = modules.time.as_ref() {
            right.push(segment);
        }
    }

    (join(&left, connector), join(&right, connector))
}

fn join(parts: &[&Segment], connector: &Segment) -> Joined {
    let mut rendered = String::new();
    let mut width = 0;
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            rendered.push_str(&connector.rendered);
            width += connector.visible_width;
        }
        rendered.push_str(&part.rendered);
        width += part.visible_width;
    }
    Joined { rendered, width }
}

fn fits(
    config: &Config,
    columns: usize,
    modules: &RenderedModules,
    selection: &Selection,
    connector: &Segment,
) -> bool {
    let (left, right) = joined_sides(modules, selection, connector);
    let (left_margin, right_margin) = effective_margins(config, columns);
    let safe_width = columns.max(1).saturating_sub(1);
    left_margin + right_margin + left.width + right.width <= safe_width
}

fn first_line(config: &Config, columns: usize, left: &Joined, right: &Joined) -> String {
    let (left_margin, right_margin) = effective_margins(config, columns);
    let mut result = " ".repeat(left_margin);
    result.push_str(&left.rendered);

    // A fill is meaningful only when there is a right-side module. Without a
    // right side, trailing fill would be invisible but could trigger terminal
    // autowrap at the last column.
    if right.width > 0 {
        let safe_width = columns.max(1).saturating_sub(1);
        let occupied = left_margin + right_margin + left.width + right.width;
        let fill = safe_width.saturating_sub(occupied);
        result.push_str(&" ".repeat(fill));
        result.push_str(&right.rendered);
        result.push_str(&" ".repeat(right_margin));
    }
    result
}

fn effective_margins(config: &Config, columns: usize) -> (usize, usize) {
    let safe_width = columns.max(1).saturating_sub(1);
    let left = config.layout.left_margin.min(safe_width);
    let right = config
        .layout
        .right_margin
        .min(safe_width.saturating_sub(left));
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::{first_line, join};
    use crate::segment::Segment;
    use crate::width::visible_width;

    #[test]
    fn connectors_are_only_inserted_between_segments() {
        let connector = Segment {
            rendered: "󰜥".to_owned(),
            visible_width: 1,
        };
        let first = Segment {
            rendered: "a".to_owned(),
            visible_width: 1,
        };
        let second = Segment {
            rendered: "b".to_owned(),
            visible_width: 1,
        };
        let joined = join(&[&first, &second], &connector);
        assert_eq!(joined.rendered, "a󰜥b");
        assert_eq!(join(&[], &connector).rendered, "");
    }

    #[test]
    fn fill_reserves_the_last_terminal_column() {
        let left = super::Joined {
            rendered: "left".to_owned(),
            width: 4,
        };
        let right = super::Joined {
            rendered: "right".to_owned(),
            width: 5,
        };
        let config = crate::config::Config::default();
        let line = first_line(&config, 20, &left, &right);
        assert!(visible_width(&line) < 20);
        assert_eq!(visible_width(&line), 19);
    }

    #[test]
    fn prompt_snapshots_stay_two_lines_at_requested_widths() {
        let config = crate::config::Config::default();
        let state = crate::prompt::PromptState {
            status: 0,
            pipestatus: vec![0],
            duration_ms: Some(186),
            command_seen: true,
        };
        for columns in [30, 40, 60, 80, 120, 160] {
            let output = super::render_full(&config, &state, columns);
            let lines: Vec<&str> = output.split('\n').collect();
            assert_eq!(lines.len(), 2, "unexpected line count at {columns}");
            assert!(
                visible_width(lines[0]) < columns,
                "first line can autowrap at {columns}: {}",
                visible_width(lines[0])
            );
            assert_eq!(
                lines[0].matches('').count(),
                lines[0].matches('').count(),
                "unbalanced capsule at {columns}"
            );
        }
    }
}
