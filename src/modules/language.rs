use crate::config::ColorPair;
use crate::segment::{capsule, Segment};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

struct Language {
    icon: &'static str,
    weight: i32,
    triggers: &'static [&'static str],
    extensions: &'static [&'static str],
}

const LANGUAGES: &[Language] = &[
    Language {
        icon: "󰌠",
        weight: 100,
        triggers: &["requirements.txt", "pyproject.toml", "setup.py", "Pipfile"],
        extensions: &["py"],
    },
    Language {
        icon: "",
        weight: 95,
        triggers: &["go.mod"],
        extensions: &["go"],
    },
    Language {
        icon: "",
        weight: 90,
        triggers: &["CMakeLists.txt", "meson.build"],
        extensions: &["cpp", "hpp", "cc", "cxx"],
    },
    Language {
        icon: "",
        weight: 88,
        triggers: &["tsconfig.json"],
        extensions: &["ts", "tsx"],
    },
    Language {
        icon: "",
        weight: 86,
        triggers: &["package.json"],
        extensions: &["js", "jsx", "mjs", "cjs"],
    },
    Language {
        icon: "",
        weight: 85,
        triggers: &["pom.xml", "build.gradle"],
        extensions: &["java"],
    },
    Language {
        icon: "",
        weight: 80,
        triggers: &[],
        extensions: &["c", "h"],
    },
    Language {
        icon: "",
        weight: 78,
        triggers: &["qmldir"],
        extensions: &["qml"],
    },
    Language {
        icon: "",
        weight: 76,
        triggers: &["Cargo.toml"],
        extensions: &["rs"],
    },
    Language {
        icon: "",
        weight: 74,
        triggers: &[],
        extensions: &["vue"],
    },
    Language {
        icon: "",
        weight: 72,
        triggers: &[],
        extensions: &["svelte"],
    },
    Language {
        icon: "",
        weight: 70,
        triggers: &["build.zig"],
        extensions: &["zig"],
    },
    Language {
        icon: "",
        weight: 65,
        triggers: &[".luarc.json"],
        extensions: &["lua"],
    },
    Language {
        icon: "",
        weight: 62,
        triggers: &[],
        extensions: &["sh", "bash"],
    },
    Language {
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
        icon: "",
        weight: 54,
        triggers: &["flake.nix", "shell.nix", "default.nix"],
        extensions: &["nix"],
    },
    Language {
        icon: "",
        weight: 52,
        triggers: &["CMakeLists.txt"],
        extensions: &["cmake"],
    },
    Language {
        icon: "󰔷",
        weight: 50,
        triggers: &["meson.build"],
        extensions: &[],
    },
    Language {
        icon: "",
        weight: 45,
        triggers: &["Justfile", "justfile", ".justfile"],
        extensions: &["just"],
    },
];

#[derive(Clone, Copy)]
struct Match {
    index: usize,
    score: i32,
}

pub fn render(colors: ColorPair) -> Option<Segment> {
    let language = detect()?;
    Some(capsule(colors, language.icon))
}

struct LanguagesFallback {
    icon: &'static str,
}

fn detect() -> Option<LanguagesFallback> {
    let current = env::current_dir().ok()?;
    let mut matches: HashMap<usize, i32> = HashMap::new();
    let mut directory = Some(current.as_path());
    let mut depth = 0;

    // Each directory is bounded independently, and the ancestor walk stops
    // after eight levels. This catches a project marker without recursively
    // scanning a large repository or filesystem.
    while let Some(path) = directory {
        let has_project_marker = scan_directory(path, depth, &mut matches);
        depth += 1;
        if has_project_marker {
            break;
        }
        if depth >= 8 {
            break;
        }
        directory = path.parent();
    }

    let best = matches
        .into_iter()
        .map(|(index, score)| Match { index, score })
        .max_by(|left, right| {
            left.score
                .cmp(&right.score)
                .then_with(|| {
                    LANGUAGES[right.index]
                        .weight
                        .cmp(&LANGUAGES[left.index].weight)
                })
                .then_with(|| right.index.cmp(&left.index))
        })?;
    Some(LanguagesFallback {
        icon: LANGUAGES[best.index].icon,
    })
}

fn scan_directory(path: &Path, depth: i32, matches: &mut HashMap<usize, i32>) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    let mut has_project_marker = false;
    for (count, entry) in entries.take(256).enumerate() {
        let Ok(entry) = entry else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let extension = Path::new(&name)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        for (index, language) in LANGUAGES.iter().enumerate() {
            let trigger = language.triggers.iter().any(|value| *value == name);
            has_project_marker |= trigger;
            let extension_match = language.extensions.iter().any(|value| *value == extension);
            if trigger || extension_match {
                let score =
                    language.weight + i32::from(extension_match) * 100 + i32::from(trigger) * 30
                        - depth * 2;
                let slot = matches.entry(index).or_insert(i32::MIN);
                *slot = (*slot).max(score);
            }
        }
        // Keep the bound explicit even if the iterator implementation changes.
        if count >= 255 {
            break;
        }
    }
    has_project_marker
}

#[cfg(test)]
mod tests {
    use super::LANGUAGES;

    #[test]
    fn preserves_the_old_icon_inventory() {
        assert!(LANGUAGES.iter().any(|language| language.icon == "󰌠"));
        assert!(LANGUAGES.iter().any(|language| language.icon == ""));
        assert!(LANGUAGES.iter().any(|language| language.icon == ""));
        assert!(LANGUAGES.iter().any(|language| language.icon == ""));
    }
}
