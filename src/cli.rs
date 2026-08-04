use crate::config;
use crate::init;
use crate::prompt::{self, PromptState, Target};
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run<I>(arguments: I) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    let arguments: Vec<String> = arguments.into_iter().collect();
    let Some(command) = arguments.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };

    match command {
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "--version" | "-V" => {
            println!("clavis-fish-theme {VERSION}");
            Ok(())
        }
        "init" => run_init(&arguments[1..]),
        "prompt" => run_prompt(&arguments[1..]),
        _ => Err(format!("unknown command `{command}` (try --help)")),
    }
}

fn run_init(arguments: &[String]) -> Result<(), String> {
    match arguments {
        [shell] if shell == "fish" => init::print_fish().map_err(|error| error.to_string()),
        [shell] => Err(format!(
            "unsupported shell `{shell}`; only `init fish` is supported"
        )),
        [] => Err("missing shell; use `clavis-fish-theme init fish`".to_owned()),
        _ => Err("init fish does not accept additional arguments".to_owned()),
    }
}

struct PromptArguments {
    target: Target,
    state: PromptState,
    columns: usize,
}

fn run_prompt(arguments: &[String]) -> Result<(), String> {
    if arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        print_prompt_help();
        return Ok(());
    }
    let parsed = parse_prompt_arguments(arguments)?;
    let configuration = config::load();
    print!(
        "{}",
        prompt::render(&configuration, parsed.target, &parsed.state, parsed.columns,)
    );
    Ok(())
}

fn parse_prompt_arguments(arguments: &[String]) -> Result<PromptArguments, String> {
    let mut target = Target::Full;
    let mut status = 0;
    let mut pipestatus = Vec::new();
    let mut duration_ms = None;
    let mut command_seen = None;
    let mut columns = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(80);
    let mut positional = Vec::new();
    let mut index = 0;

    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--transient-left" {
            target = Target::TransientLeft;
        } else if argument == "--transient-right" || argument == "--right" {
            target = Target::TransientRight;
        } else if let Some(value) = argument.strip_prefix("--status=") {
            status = parse_i32(value, "status")?;
        } else if argument == "--status" {
            index += 1;
            status = parse_i32(next_argument(arguments, index, "status")?, "status")?;
        } else if let Some(value) = argument
            .strip_prefix("--cmd-duration=")
            .or_else(|| argument.strip_prefix("--duration="))
        {
            duration_ms = Some(if value.is_empty() {
                0
            } else {
                parse_u64(value, "cmd-duration")?
            });
        } else if argument == "--cmd-duration" || argument == "--duration" {
            index += 1;
            let value = next_argument(arguments, index, "cmd-duration")?;
            duration_ms = Some(if value.is_empty() {
                0
            } else {
                parse_u64(value, "cmd-duration")?
            });
        } else if let Some(value) = argument
            .strip_prefix("--terminal-width=")
            .or_else(|| argument.strip_prefix("--columns="))
        {
            columns = parse_usize(value, "terminal-width")?;
        } else if let Some(value) = argument.strip_prefix("--width=") {
            columns = parse_usize(value, "width")?;
        } else if argument == "--terminal-width" || argument == "--width" || argument == "--columns"
        {
            index += 1;
            columns = parse_usize(
                next_argument(arguments, index, "terminal-width")?,
                "terminal-width",
            )?;
        } else if let Some(value) = argument.strip_prefix("--pipestatus=") {
            pipestatus = parse_pipeline_status(value);
        } else if argument == "--pipestatus" {
            index += 1;
            pipestatus = parse_pipeline_status(next_argument(arguments, index, "pipestatus")?);
        } else if let Some(value) = argument.strip_prefix("--command-seen=") {
            command_seen = Some(parse_bool(value, "command-seen")?);
        } else if argument == "--command-seen" {
            command_seen = Some(true);
        } else if argument.starts_with('-') {
            return Err(format!("unknown prompt option `{argument}`"));
        } else {
            positional.push(argument.as_str());
        }
        index += 1;
    }

    // Positional arguments are accepted as a small convenience for renderer
    // inspection: status, duration-ms, columns. Fish itself uses named flags.
    if !positional.is_empty() {
        if positional.len() > 3 {
            return Err("prompt accepts at most three positional arguments".to_owned());
        }
        status = parse_i32(positional[0], "status")?;
        if positional.len() >= 2 && !positional[1].is_empty() {
            duration_ms = Some(parse_u64(positional[1], "duration")?);
        }
        if positional.len() == 3 {
            columns = parse_usize(positional[2], "terminal-width")?;
        }
    }

    let command_seen = command_seen.unwrap_or(duration_ms.is_some());
    Ok(PromptArguments {
        target,
        state: PromptState {
            status,
            pipestatus,
            duration_ms,
            command_seen,
        },
        columns: columns.max(1),
    })
}

fn next_argument<'a>(arguments: &'a [String], index: usize, name: &str) -> Result<&'a str, String> {
    arguments
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for --{name}"))
}

fn parse_i32(value: &str, name: &str) -> Result<i32, String> {
    value
        .parse()
        .map_err(|_| format!("invalid {name} value `{value}`"))
}

fn parse_u64(value: &str, name: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| format!("invalid {name} value `{value}`"))
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| format!("invalid {name} value `{value}`"))
}

fn parse_bool(value: &str, name: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid {name} value `{value}`")),
    }
}

fn parse_pipeline_status(value: &str) -> Vec<i32> {
    value
        .split([',', ' '])
        .filter_map(|part| part.parse().ok())
        .collect()
}

fn print_help() {
    println!(
        "clavis-fish-theme {VERSION}\n\n\
Rust two-line capsule prompt for Fish 4.1+.\n\n\
USAGE:\n  clavis-fish-theme init fish\n  clavis-fish-theme prompt [OPTIONS]\n  clavis-fish-theme --version\n  clavis-fish-theme --help\n\n\
The init command prints sourceable Fish code and never edits user files."
    );
}

fn print_prompt_help() {
    println!(
        "Internal prompt renderer. Normal use is through `init fish`.\n\n\
Options: --terminal-width N --status N --pipestatus LIST\n\
         --cmd-duration MS --command-seen[=BOOL]\n\
         --transient-left | --transient-right"
    );
}
