use crate::config::ColorPair;
use crate::segment::{capsule, Segment};
use std::env;
use std::path::{Component, Path, PathBuf};

struct PathInfo {
    icon: &'static str,
    components: Vec<String>,
    home: bool,
    absolute: bool,
}

pub fn render(colors: ColorPair) -> Vec<Segment> {
    let info = path_info();
    let mut contents = Vec::new();

    let full = info.components_text(None);
    contents.push(full);

    if info.components.len() > 1 {
        for count in (1..=info.components.len().min(3)).rev() {
            contents.push(info.components_text(Some(count)));
        }
        // This is deliberately just the current directory name. The layout
        // fitter reaches it only after the ellipsis variants have failed.
        contents.push(info.components.last().cloned().unwrap_or_default());
    }

    let mut result = Vec::new();
    for text in contents {
        let content = if text.is_empty() {
            info.icon.to_owned()
        } else {
            format!("{} {text}", info.icon)
        };
        let segment = capsule(colors, &content);
        if result.last().map_or(true, |previous: &Segment| {
            previous.rendered != segment.rendered
        }) {
            result.push(segment);
        }
    }
    result
}

fn path_info() -> PathInfo {
    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("?"));
    if current == Path::new("/") {
        return PathInfo {
            icon: "󰀘",
            components: Vec::new(),
            home: false,
            absolute: true,
        };
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    if let Some(home) = home {
        if let Ok(relative) = current.strip_prefix(&home) {
            let components = normal_components(relative);
            let icon = match components.first().map(String::as_str) {
                Some("Downloads") => "",
                Some("Desktop") => "",
                Some("Documents") => "",
                Some("Pictures") => "",
                Some("Videos") => "",
                Some("Music") => "󰎈",
                Some("Templates") => "󰏪",
                Some("Public") => "󰙨",
                Some("GitHub" | "Projects" | "Workspace") => "󰊤",
                _ => "",
            };
            return PathInfo {
                icon,
                components,
                home: true,
                absolute: false,
            };
        }
    }

    PathInfo {
        icon: "",
        components: normal_components(&current),
        home: false,
        absolute: true,
    }
}

impl PathInfo {
    fn components_text(&self, tail_count: Option<usize>) -> String {
        let text = components_text(&self.components, tail_count);
        if self.home && tail_count.is_none() {
            if text.is_empty() {
                "~".to_owned()
            } else {
                format!("~/{text}")
            }
        } else if self.absolute && tail_count.is_none() && !text.is_empty() {
            format!("/{text}")
        } else {
            text
        }
    }
}

fn normal_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn components_text(components: &[String], tail_count: Option<usize>) -> String {
    if components.is_empty() {
        return String::new();
    }
    if let Some(count) = tail_count {
        let count = count.min(components.len());
        let start = components.len() - count;
        return format!("••/{}", components[start..].join("/"));
    }
    components.join("/")
}

#[cfg(test)]
mod tests {
    use super::{components_text, PathInfo};

    #[test]
    fn shortens_middle_components_without_byte_cutting() {
        let components = vec![
            "中文".to_owned(),
            "二".to_owned(),
            "三".to_owned(),
            "当前".to_owned(),
        ];
        assert_eq!(components_text(&components, None), "中文/二/三/当前");
        assert_eq!(components_text(&components, Some(2)), "••/三/当前");
    }

    #[test]
    fn home_paths_keep_the_tilde_in_the_wide_variant() {
        let info = PathInfo {
            icon: "",
            components: vec!["Projects".to_owned(), "clavis".to_owned()],
            home: true,
            absolute: false,
        };
        assert_eq!(info.components_text(None), "~/Projects/clavis");
    }
}
