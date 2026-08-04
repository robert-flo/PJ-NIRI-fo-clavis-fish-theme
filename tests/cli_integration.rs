use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
    assert_eq!(
        output.matches('\n').count(),
        2,
        "full prompt has a spacer plus two visual lines"
    );
    assert!(output.starts_with('\n'));
    assert!(output.contains("╭─ "));
    assert!(output.contains("╰─ "));
    assert!(output.contains(" "));

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
    assert!(!right_output.contains(""));
    assert!(!right_output.contains(""));
    assert!(!right_output.contains("48;2;64;71;83m"));
    assert!(right_output.contains("38;2;220;227;242m"));
}

#[test]
fn init_preserves_the_real_binary_path_in_fish() {
    let root = temporary_directory();
    let bin_dir = root
        .join("My Tools")
        .join("it's-tools")
        .join("$tools")
        .join("$(not-a-command)")
        .join(".cargo/bin");
    fs::create_dir_all(&bin_dir).expect("binary directory");
    let installed_binary = bin_dir.join("clavis-fish-theme");
    fs::copy(binary(), &installed_binary).expect("copy test binary");

    let init = Command::new(&installed_binary)
        .args(["init", "fish"])
        .output()
        .expect("init renderer");
    assert!(init.status.success());
    let script = String::from_utf8(init.stdout).expect("init script utf8");
    let expected_path = installed_binary.to_string_lossy();
    assert!(script.contains("set -g __clavis_fish_theme_binary '/tmp/"));
    assert!(script.contains("it'\\''s-tools"));
    assert!(script.contains("/$tools/$(not-a-command)/.cargo/bin/clavis-fish-theme'"));
    assert!(!script.contains("_2E"));
    assert!(!script.contains("_20"));

    let mut syntax = Command::new("fish")
        .args(["--no-config", "-n"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("fish syntax check");
    syntax
        .stdin
        .take()
        .expect("fish stdin")
        .write_all(script.as_bytes())
        .expect("write init script");
    let syntax_output = syntax.wait_with_output().expect("fish syntax result");
    assert!(
        syntax_output.status.success(),
        "generated init script is invalid: {}",
        String::from_utf8_lossy(&syntax_output.stderr)
    );

    let fish_code = concat!(
        "clavis-fish-theme init fish | source; ",
        "clavis-fish-theme init fish | source; ",
        "printf '%s\\n' \"$__clavis_fish_theme_binary\"; ",
        "\"$__clavis_fish_theme_binary\" --version"
    );
    let fish = Command::new("fish")
        .args(["--no-config", "--private", "--interactive", "-c", fish_code])
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("TERM", "dumb")
        .env("PATH", format!("{}:/usr/bin:/bin", bin_dir.display()))
        .output()
        .expect("fish integration");
    assert!(
        fish.status.success(),
        "Fish could not source init: {}",
        String::from_utf8_lossy(&fish.stderr)
    );
    let fish_stdout = String::from_utf8_lossy(&fish.stdout);
    assert!(
        fish_stdout.lines().any(|line| line == expected_path),
        "Fish variable did not preserve path: {fish_stdout:?}"
    );
    assert!(
        fish_stdout.contains("clavis-fish-theme 0.1.0"),
        "Fish could not execute the resolved binary: {fish_stdout:?}"
    );

    fs::remove_dir_all(root).expect("cleanup");
}
