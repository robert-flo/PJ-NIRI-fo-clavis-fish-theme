use crate::config::ColorPair;
use crate::segment::{capsule, Segment};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const MAX_DEPTH: usize = 8;
const MAX_ENTRIES: usize = 2_048;
const MAX_SCAN_TIME: Duration = Duration::from_millis(10);
const IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "vendor",
    "dist",
    "build",
    ".cache",
    ".venv",
    "venv",
];

struct Language {
    name: &'static str,
    icon: &'static str,
    weight: i32,
    triggers: &'static [&'static str],
    extensions: &'static [&'static str],
}

const LANGUAGES: &[Language] = &[
    Language {
        name: "Python",
        icon: "󰌠",
        weight: 100,
        triggers: &["requirements.txt", "pyproject.toml", "setup.py", "Pipfile"],
        extensions: &["py"],
    },
    Language {
        name: "Go",
        icon: "",
        weight: 95,
        triggers: &["go.mod"],
        extensions: &["go"],
    },
    Language {
        name: "C++",
        icon: "",
        weight: 90,
        triggers: &["CMakeLists.txt", "meson.build"],
        extensions: &["cpp", "hpp", "cc", "cxx"],
    },
    Language {
        name: "TypeScript",
        icon: "",
        weight: 88,
        triggers: &["tsconfig.json"],
        extensions: &["ts", "tsx"],
    },
    Language {
        name: "JavaScript",
        icon: "",
        weight: 86,
        triggers: &["package.json"],
        extensions: &["js", "jsx", "mjs", "cjs"],
    },
    Language {
        name: "Java",
        icon: "",
        weight: 85,
        triggers: &["pom.xml", "build.gradle"],
        extensions: &["java"],
    },
    Language {
        name: "C",
        icon: "",
        weight: 80,
        triggers: &[],
        extensions: &["c", "h"],
    },
    Language {
        name: "QML",
        icon: "",
        weight: 78,
        triggers: &["qmldir"],
        extensions: &["qml"],
    },
    Language {
        name: "Rust",
        icon: "",
        weight: 76,
        triggers: &["Cargo.toml"],
        extensions: &["rs"],
    },
    Language {
        name: "Vue",
        icon: "",
        weight: 74,
        triggers: &[],
        extensions: &["vue"],
    },
    Language {
        name: "Svelte",
        icon: "",
        weight: 72,
        triggers: &[],
        extensions: &["svelte"],
    },
    Language {
        name: "Zig",
        icon: "",
        weight: 70,
        triggers: &["build.zig"],
        extensions: &["zig"],
    },
    Language {
        name: "Lua",
        icon: "",
        weight: 65,
        triggers: &[".luarc.json"],
        extensions: &["lua"],
    },
    Language {
        name: "Shell",
        icon: "",
        weight: 62,
        triggers: &[],
        extensions: &["sh", "bash", "fish"],
    },
    Language {
        name: "Docker",
        icon: "",
        weight: 55,
        triggers: &[
            "Dockerfile",
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ],
        extensions: &[],
    },
    Language {
        name: "Nix",
        icon: "",
        weight: 54,
        triggers: &["flake.nix", "shell.nix", "default.nix"],
        extensions: &["nix"],
    },
    Language {
        name: "CMake",
        icon: "",
        weight: 52,
        triggers: &["CMakeLists.txt"],
        extensions: &["cmake"],
    },
    Language {
        name: "Meson",
        icon: "󰔷",
        weight: 50,
        triggers: &["meson.build"],
        extensions: &[],
    },
    Language {
        name: "Just",
        icon: "",
        weight: 45,
        triggers: &["Justfile", "justfile", ".justfile"],
        extensions: &["just"],
    },
];

#[derive(Clone, Copy, Default)]
struct Match {
    index: usize,
    source_files: usize,
    marker: bool,
}

pub fn render(colors: ColorPair) -> Option<Segment> {
    let languages = detect()?;
    Some(render_detected(colors, &languages))
}

fn render_detected(colors: ColorPair, languages: &[&Language]) -> Segment {
    let icons = languages
        .iter()
        .map(|language| language.icon)
        .collect::<Vec<_>>()
        .join(" ");
    capsule(colors, &format!(" {icons} "))
}

fn detect() -> Option<Vec<&'static Language>> {
    let current = env::current_dir().ok()?;
    detect_from(&current)
}

fn detect_from(current: &Path) -> Option<Vec<&'static Language>> {
    let root = project_root(current);
    let mut evidence = vec![Match::default(); LANGUAGES.len()];
    for (index, item) in evidence.iter_mut().enumerate() {
        item.index = index;
    }
    scan_project(&root, &mut evidence);
    evidence.retain(|item| item.source_files > 0 || item.marker);
    evidence.sort_by(|left, right| {
        right
            .source_files
            .cmp(&left.source_files)
            .then_with(|| right.marker.cmp(&left.marker))
            .then_with(|| {
                LANGUAGES[right.index]
                    .weight
                    .cmp(&LANGUAGES[left.index].weight)
            })
            .then_with(|| LANGUAGES[left.index].name.cmp(LANGUAGES[right.index].name))
    });
    let result = evidence
        .into_iter()
        .take(3)
        .map(|item| &LANGUAGES[item.index])
        .collect::<Vec<_>>();
    (!result.is_empty()).then_some(result)
}

fn project_root(current: &Path) -> PathBuf {
    for directory in current.ancestors().take(MAX_DEPTH) {
        if directory.join(".git").exists() || contains_project_marker(directory) {
            return directory.to_path_buf();
        }
    }
    current.to_path_buf()
}

fn contains_project_marker(directory: &Path) -> bool {
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    entries.take(256).filter_map(Result::ok).any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        LANGUAGES
            .iter()
            .any(|language| language.triggers.iter().any(|trigger| *trigger == name))
    })
}

fn scan_project(root: &Path, evidence: &mut [Match]) {
    let started = Instant::now();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    let mut inspected = 0;

    while let Some((directory, depth)) = queue.pop_front() {
        if inspected >= MAX_ENTRIES || started.elapsed() >= MAX_SCAN_TIME {
            break;
        }
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if inspected >= MAX_ENTRIES || started.elapsed() >= MAX_SCAN_TIME {
                return;
            }
            inspected += 1;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if depth < MAX_DEPTH && !IGNORED_DIRECTORIES.contains(&name.as_ref()) {
                    queue.push_back((entry.path(), depth + 1));
                }
                continue;
            }
            if !file_type.is_file() {
                continue;
            }

            let extension = Path::new(name.as_ref())
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            for (index, language) in LANGUAGES.iter().enumerate() {
                if language.triggers.iter().any(|trigger| *trigger == name) {
                    evidence[index].marker = true;
                }
                if language
                    .extensions
                    .iter()
                    .any(|candidate| *candidate == extension)
                {
                    evidence[index].source_files += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_from, render_detected, LANGUAGES};
    use crate::config::Config;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("clavis-language-{suffix}"));
        fs::create_dir_all(&path).expect("temporary directory");
        path
    }

    #[test]
    fn preserves_the_old_icon_inventory() {
        assert!(LANGUAGES.iter().any(|language| language.icon == "󰌠"));
        assert!(LANGUAGES.iter().any(|language| language.icon == ""));
        assert!(LANGUAGES.iter().any(|language| language.icon == ""));
        assert!(LANGUAGES.iter().any(|language| language.icon == ""));
    }

    #[test]
    fn centers_up_to_three_icons_with_explicit_spacing() {
        let rust = LANGUAGES
            .iter()
            .find(|language| language.name == "Rust")
            .expect("Rust");
        let python = LANGUAGES
            .iter()
            .find(|language| language.name == "Python")
            .expect("Python");
        let shell = LANGUAGES
            .iter()
            .find(|language| language.name == "Shell")
            .expect("Shell");
        let segment = render_detected(Config::default().colors.language, &[rust, python, shell]);
        assert!(segment.rendered.contains("  󰌠  "));
    }

    #[test]
    fn ranks_three_languages_from_the_project_root() {
        let root = temporary_directory();
        fs::create_dir_all(root.join(".git")).expect("git marker");
        fs::create_dir_all(root.join("src/nested")).expect("source directory");
        fs::create_dir_all(root.join("target/generated")).expect("ignored directory");
        fs::write(root.join("Cargo.toml"), "[package]\nname='fixture'\n").expect("marker");
        for name in ["one.rs", "two.rs", "three.rs", "four.rs"] {
            fs::write(root.join("src").join(name), "").expect("rust source");
        }
        for name in ["one.py", "two.py", "three.py"] {
            fs::write(root.join("src").join(name), "").expect("python source");
        }
        for name in ["one.fish", "two.fish"] {
            fs::write(root.join("src").join(name), "").expect("fish source");
        }
        for index in 0..20 {
            fs::write(
                root.join("target/generated").join(format!("{index}.ts")),
                "",
            )
            .expect("ignored TypeScript source");
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("target/generated"), root.join("src/linked"))
            .expect("directory symlink");

        let languages = detect_from(&root.join("src/nested")).expect("detected languages");
        let names = languages
            .iter()
            .map(|language| language.name)
            .collect::<Vec<_>>();
        assert_eq!(names, ["Rust", "Python", "Shell"]);

        fs::remove_dir_all(root).expect("cleanup");
    }
}
