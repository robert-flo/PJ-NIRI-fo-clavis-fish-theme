use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_clavis-fish-theme")
}

fn temporary_directory() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("clavis-cli-{suffix}"));
    fs::create_dir_all(&path).expect("temporary directory");
    path
}

#[test]
fn config_layers_reload_without_creating_files() {
    let root = temporary_directory();
    let config_dir = root.join("xdg");
    let command = |args: &[&str]| {
        Command::new(binary())
            .args(args)
            .env("XDG_CONFIG_HOME", &config_dir)
            .output()
            .expect("renderer")
    };

    let first = command(&[
        "prompt",
        "--terminal-width=160",
        "--status=0",
        "--cmd-duration=0",
        "--command-seen=1",
    ]);
    assert!(first.status.success());
    assert!(!config_dir.exists(), "prompt must not create XDG config");

    fs::create_dir_all(config_dir.join("clavis-fish-theme")).expect("config directory");
    let theme_dir = config_dir.join("clavis-fish-theme");
    fs::write(
        theme_dir.join("colors.conf"),
        "[colors]\npath_background=#010203\npath_foreground=#abcdef\n",
    )
    .expect("colors");
    fs::write(
        theme_dir.join("config.conf"),
        "[modules]\ngit=false\ntime=false\n[colors]\npath_background=#040506\n",
    )
    .expect("config");

    let overridden = command(&[
        "prompt",
        "--terminal-width=160",
        "--status=0",
        "--cmd-duration=186",
        "--command-seen=1",
    ]);
    let output = String::from_utf8(overridden.stdout).expect("utf8 prompt");
    assert!(output.contains("48;2;4;5;6m"), "config.conf color wins");
    assert!(
        output.contains("38;2;171;205;239m"),
        "colors.conf field survives"
    );
    assert!(!output.contains(" "), "disabled time is not rendered");
    assert_eq!(output.matches('\n').count(), 1, "full prompt has two lines");

    fs::write(
        theme_dir.join("colors.conf"),
        "[colors]\npath_background=#111213\npath_foreground=#141516\n",
    )
    .expect("updated colors");
    let reloaded = command(&[
        "prompt",
        "--terminal-width=160",
        "--status=0",
        "--cmd-duration=186",
        "--command-seen=1",
    ]);
    let reloaded_output = String::from_utf8(reloaded.stdout).expect("utf8 prompt");
    assert!(
        reloaded_output.contains("48;2;4;5;6m"),
        "explicit config color remains highest priority"
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn transient_targets_keep_their_separate_contracts() {
    let left = Command::new(binary())
        .args([
            "prompt",
            "--transient-left",
            "--terminal-width=80",
            "--status=1",
        ])
        .output()
        .expect("left renderer");
    let left_output = String::from_utf8(left.stdout).expect("left utf8");
    assert!(left_output.contains(" "));
    assert!(left_output.contains("38;2;255;218;214m"));
    assert!(!left_output.contains(""));

    let right = Command::new(binary())
        .args(["prompt", "--transient-right", "--terminal-width=80"])
        .output()
        .expect("right renderer");
    let right_output = String::from_utf8(right.stdout).expect("right utf8");
    assert!(right_output.contains(" "));
    assert!(right_output.contains(""));
}
