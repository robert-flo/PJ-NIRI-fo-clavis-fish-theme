use crate::color::Color;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorPair {
    pub background: Color,
    pub foreground: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub path: ColorPair,
    pub git: ColorPair,
    pub language: ColorPair,
    pub duration: ColorPair,
    pub time: ColorPair,
    pub error: ColorPair,
    pub connector: Color,
    pub arrow: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modules {
    pub path: bool,
    pub git: bool,
    pub language: bool,
    pub duration: bool,
    pub time: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutConfig {
    pub left_margin: usize,
    pub right_margin: usize,
    pub path_max_width: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub colors: Palette,
    pub modules: Modules,
    pub layout: LayoutConfig,
}

const DEFAULT_PATH_BACKGROUND: Color = Color::rgb(0xc0, 0xc7, 0xd5);
const DEFAULT_PATH_FOREGROUND: Color = Color::rgb(0x2a, 0x31, 0x3c);
const DEFAULT_GIT_BACKGROUND: Color = Color::rgb(0x43, 0x47, 0x4e);
const DEFAULT_GIT_FOREGROUND: Color = Color::rgb(0xdf, 0xe2, 0xeb);
const DEFAULT_LANGUAGE_BACKGROUND: Color = Color::rgb(0x3c, 0x47, 0x58);
const DEFAULT_LANGUAGE_FOREGROUND: Color = Color::rgb(0xd8, 0xe3, 0xf8);
const DEFAULT_DURATION_BACKGROUND: Color = Color::rgb(0x49, 0x45, 0x4f);
const DEFAULT_DURATION_FOREGROUND: Color = Color::rgb(0xe6, 0xe1, 0xe5);
const DEFAULT_TIME_BACKGROUND: Color = Color::rgb(0x40, 0x47, 0x53);
const DEFAULT_TIME_FOREGROUND: Color = Color::rgb(0xdc, 0xe3, 0xf2);
const DEFAULT_ERROR_BACKGROUND: Color = Color::rgb(0x93, 0x00, 0x0a);
const DEFAULT_ERROR_FOREGROUND: Color = Color::rgb(0xff, 0xda, 0xd6);
const DEFAULT_CONNECTOR: Color = Color::rgb(0x91, 0x90, 0x92);
const DEFAULT_ARROW: Color = Color::rgb(0xc0, 0xc7, 0xd5);
const DEFAULT_PATH_MAX_WIDTH: usize = 40;

impl Default for Palette {
    fn default() -> Self {
        Self {
            path: ColorPair {
                background: DEFAULT_PATH_BACKGROUND,
                foreground: DEFAULT_PATH_FOREGROUND,
            },
            git: ColorPair {
                background: DEFAULT_GIT_BACKGROUND,
                foreground: DEFAULT_GIT_FOREGROUND,
            },
            language: ColorPair {
                background: DEFAULT_LANGUAGE_BACKGROUND,
                foreground: DEFAULT_LANGUAGE_FOREGROUND,
            },
            duration: ColorPair {
                background: DEFAULT_DURATION_BACKGROUND,
                foreground: DEFAULT_DURATION_FOREGROUND,
            },
            time: ColorPair {
                background: DEFAULT_TIME_BACKGROUND,
                foreground: DEFAULT_TIME_FOREGROUND,
            },
            error: ColorPair {
                background: DEFAULT_ERROR_BACKGROUND,
                foreground: DEFAULT_ERROR_FOREGROUND,
            },
            connector: DEFAULT_CONNECTOR,
            arrow: DEFAULT_ARROW,
        }
    }
}

impl Default for Modules {
    fn default() -> Self {
        Self {
            path: true,
            git: true,
            language: true,
            duration: true,
            time: true,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            left_margin: 0,
            right_margin: 0,
            path_max_width: DEFAULT_PATH_MAX_WIDTH,
        }
    }
}

#[derive(Default)]
struct IniValues {
    values: HashMap<(String, String), String>,
}

impl IniValues {
    fn read(path: &Path) -> Self {
        let Ok(content) = fs::read_to_string(path) else {
            return Self::default();
        };
        Self::parse(&content)
    }

    fn parse(content: &str) -> Self {
        let mut result = Self::default();
        let mut section = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            if !section.is_empty() && !key.is_empty() {
                result
                    .values
                    .insert((section.clone(), key), value.trim().to_owned());
            }
        }
        result
    }

    fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.values
            .get(&(section.to_owned(), key.to_owned()))
            .map(String::as_str)
    }
}

pub fn config_directory() -> PathBuf {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(xdg).join("clavis-fish-theme");
    }
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("clavis-fish-theme")
}

pub fn load() -> Config {
    load_from_directory(&config_directory())
}

pub fn load_from_directory(directory: &Path) -> Config {
    let mut config = Config::default();
    let colors = IniValues::read(&directory.join("colors.conf"));
    apply_colors(&colors, &mut config.colors);

    // This is intentionally a separate pass: config.conf does not make
    // colors.conf disappear from the precedence chain.
    let user = IniValues::read(&directory.join("config.conf"));
    apply_colors(&user, &mut config.colors);
    apply_modules(&user, &mut config.modules);
    apply_layout(&user, &mut config.layout);
    config
}

fn apply_color(values: &IniValues, key: &str, target: &mut Color) {
    if let Some(value) = values.get("colors", key) {
        if let Some(parsed) = Color::parse(value) {
            *target = parsed;
        } else if debug_enabled() {
            eprintln!("clavis-fish-theme: ignoring invalid color {key}={value}");
        }
    }
}

fn apply_colors(values: &IniValues, palette: &mut Palette) {
    apply_color(values, "path_background", &mut palette.path.background);
    apply_color(values, "path_foreground", &mut palette.path.foreground);
    apply_color(values, "git_background", &mut palette.git.background);
    apply_color(values, "git_foreground", &mut palette.git.foreground);
    apply_color(
        values,
        "language_background",
        &mut palette.language.background,
    );
    apply_color(
        values,
        "language_foreground",
        &mut palette.language.foreground,
    );
    apply_color(
        values,
        "duration_background",
        &mut palette.duration.background,
    );
    apply_color(
        values,
        "duration_foreground",
        &mut palette.duration.foreground,
    );
    apply_color(values, "time_background", &mut palette.time.background);
    apply_color(values, "time_foreground", &mut palette.time.foreground);
    apply_color(values, "error_background", &mut palette.error.background);
    apply_color(values, "error_foreground", &mut palette.error.foreground);
    apply_color(values, "connector", &mut palette.connector);
    apply_color(values, "arrow", &mut palette.arrow);
}

fn apply_modules(values: &IniValues, modules: &mut Modules) {
    for (key, target) in [
        ("path", &mut modules.path),
        ("git", &mut modules.git),
        ("language", &mut modules.language),
        ("duration", &mut modules.duration),
        ("time", &mut modules.time),
    ] {
        if let Some(value) = values.get("modules", key) {
            if let Some(parsed) = parse_bool(value) {
                *target = parsed;
            } else if debug_enabled() {
                eprintln!("clavis-fish-theme: ignoring invalid boolean {key}={value}");
            }
        }
    }
}

fn apply_layout(values: &IniValues, layout: &mut LayoutConfig) {
    for (key, target) in [
        ("left_margin", &mut layout.left_margin),
        ("right_margin", &mut layout.right_margin),
    ] {
        if let Some(value) = values.get("layout", key) {
            match value.parse::<i64>() {
                Ok(value) => *target = value.clamp(0, 16) as usize,
                Err(_) if debug_enabled() => {
                    eprintln!("clavis-fish-theme: ignoring invalid margin {key}={value}");
                }
                Err(_) => {}
            }
        }
    }

    if let Some(value) = values.get("layout", "path_max_width") {
        match value.parse::<i64>() {
            Ok(value) => layout.path_max_width = value.clamp(8, 160) as usize,
            Err(_) if debug_enabled() => {
                eprintln!("clavis-fish-theme: ignoring invalid layout path_max_width={value}");
            }
            Err(_) => {}
        }
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn debug_enabled() -> bool {
    matches!(
        env::var("CLAVIS_FISH_THEME_DEBUG").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

#[cfg(test)]
mod tests {
    use super::{load_from_directory, Config};
    use crate::color::Color;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("clavis-config-{suffix}"));
        fs::create_dir_all(&path).expect("temporary config directory");
        path
    }

    #[test]
    fn applies_both_color_layers_and_clamps_layout() {
        let directory = temporary_directory();
        fs::write(
            directory.join("colors.conf"),
            "[colors]\npath_background=#abcdef\npath_foreground=#123456\n",
        )
        .expect("colors");
        fs::write(
            directory.join("config.conf"),
            "[modules]\ngit=false\n[layout]\nleft_margin=99\nright_margin=-4\npath_max_width=999\n[colors]\npath_background=#010203\nconnector=nope\n",
        )
        .expect("config");

        let config = load_from_directory(&directory);
        assert_eq!(config.colors.path.background, Color::rgb(1, 2, 3));
        assert_eq!(config.colors.path.foreground, Color::rgb(0x12, 0x34, 0x56));
        assert_eq!(config.colors.connector, Config::default().colors.connector);
        assert!(!config.modules.git);
        assert_eq!(config.layout.left_margin, 16);
        assert_eq!(config.layout.right_margin, 0);
        assert_eq!(config.layout.path_max_width, 160);

        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn missing_files_do_not_create_configuration() {
        let directory = temporary_directory();
        let config = load_from_directory(&directory);
        assert_eq!(config.layout.path_max_width, 40);
        assert_eq!(fs::read_dir(&directory).expect("directory").count(), 0);
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn invalid_path_width_keeps_the_builtin_default() {
        let directory = temporary_directory();
        fs::write(
            directory.join("config.conf"),
            "[layout]\npath_max_width=wide\n",
        )
        .expect("config");
        assert_eq!(load_from_directory(&directory).layout.path_max_width, 40);
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn checked_in_color_example_covers_the_builtin_palette() {
        let values = super::IniValues::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/colors.conf"
        )));
        let keys = [
            "path_background",
            "path_foreground",
            "git_background",
            "git_foreground",
            "language_background",
            "language_foreground",
            "duration_background",
            "duration_foreground",
            "time_background",
            "time_foreground",
            "error_background",
            "error_foreground",
            "connector",
            "arrow",
        ];
        for key in keys {
            assert!(
                values.get("colors", key).is_some(),
                "missing example key {key}"
            );
        }
        let mut palette = super::Palette::default();
        super::apply_colors(&values, &mut palette);
        assert_eq!(palette, super::Palette::default());
    }
}
