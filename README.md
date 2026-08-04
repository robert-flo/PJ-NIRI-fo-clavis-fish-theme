# clavis-fish-theme

`clavis-fish-theme` is a Rust two-line capsule prompt for Fish 4.1+. It keeps
the visual language of the original Clavis prompt—Nerd Font half-capsules,
Powerline separators, duration/path/Git/language/time modules—and rebuilds the
runtime around one small Fish-only binary.

It supports:

- native Fish transient prompt rendering (`--final-rendering`);
- Material 3/Matugen palettes with hot reload on the next prompt;
- ANSI-aware Unicode cell-width calculation;
- bounded language detection and a timed, single Git status query.

The package does not edit `config.fish`, create a first-run configuration,
install shell files, run a daemon, or support another shell. This is a Fish
rewrite of the previous shell theme; the old command-driven configuration and
Zsh lifecycle are intentionally not part of this package.

## Requirements

- Rust and Cargo;
- Fish 4.1 or newer;
- a Nerd Font containing the Powerline half-circles and the module icons;
- Git when the Git module is enabled.

## Build from source

```bash
git clone https://github.com/Archirithm/clavis-fish-theme.git
cd clavis-fish-theme
cargo build --release --locked
```

The release profile uses thin LTO and strips the single executable. Development
builds use the ordinary Cargo workflow:

```bash
cargo build
```

## Install from source

```bash
cargo install --path . --locked
```

The binary is normally installed at `~/.cargo/bin/clavis-fish-theme`; make
sure `~/.cargo/bin` is on `PATH`. Cargo installs only this one binary. The Fish
initialization source is embedded in it with `include_str!`.

## Enable

Add this line manually to `~/.config/fish/config.fish`:

```fish
if status is-interactive
    clavis-fish-theme init fish | source
end
```

`init fish` prints complete sourceable Fish code. It resolves and embeds the
currently running executable path at initialization time, safely escapes that
path for Fish, installs only private `__clavis_fish_theme_*` state/event names,
and sets:

```fish
set -g fish_transient_prompt 1
```

It is safe to source the line repeatedly: the package event handler is removed
and registered once per source. The only ordinary Fish functions it defines
are the two prompt entry points that Fish itself calls, `fish_prompt` and
`fish_right_prompt`; unrelated user functions are not removed.

The installer never changes `config.fish` and never creates the configuration
directory.

## Uninstall

First remove the initialization line from `config.fish`, then run:

```bash
cargo uninstall clavis-fish-theme
```

Uninstall does not remove
`~/.config/clavis-fish-theme/`, so user palettes and settings remain
recoverable.

## Configuration

The optional files live in:

```text
${XDG_CONFIG_HOME:-$HOME/.config}/clavis-fish-theme/
```

Supported files are `config.conf` and `colors.conf`. With neither file
present, all built-in defaults are active. The renderer reads these two small
files on every prompt invocation, so a Matugen palette change is visible on
the next prompt without restarting Fish.

Example `config.conf`:

```ini
[modules]
path=true
git=true
language=true
duration=true
time=true

[layout]
left_margin=0
right_margin=0
path_max_width=40

[colors]
# Optional user overrides:
# path_background=#c0c7d5
# path_foreground=#2a313c
```

`left_margin` and `right_margin` are terminal character cells, default to zero,
and are clamped to `0..=16`. They apply to both the ordinary and transient
prompt. They are not terminal emulator window padding.

`path_max_width` limits the complete path capsule in terminal cells, including
its half-circles and directory icon. It defaults to `40` and is clamped to
`8..=160`; long paths first drop parent components and then truncate a single
long directory name on a Unicode grapheme boundary with `…`.

Color precedence is strict:

```text
Rust built-in defaults < colors.conf < config.conf [colors]
```

Only valid `#RRGGBB` colors, booleans, and margins override the preceding
layer. Missing or invalid values fall back field-by-field; malformed values
never produce a broken ANSI sequence. Prompt stdout contains only rendered
prompt text. Set `CLAVIS_FISH_THEME_DEBUG=1` when developing if field-level
fallback diagnostics are needed on stderr.

The built-in palette is:

```ini
[colors]
path_background=#c0c7d5
path_foreground=#2a313c
git_background=#43474e
git_foreground=#dfe2eb
language_background=#3c4758
language_foreground=#d8e3f8
duration_background=#49454f
duration_foreground=#e6e1e5
time_background=#404753
time_foreground=#dce3f2
error_background=#93000a
error_foreground=#ffdad6
connector=#919092
arrow=#c0c7d5
```

Copyable examples are in `examples/config.conf` and
`examples/colors.conf`; they are documentation assets and are not installed by
Cargo.

## Matugen

`examples/matugen.conf` is a complete Matugen template. With the local Matugen
4.1.0 release, `surface_variant` and `on_surface_variant` are valid Material 3
tokens and are used for the duration capsule. If a future Matugen build
renames those tokens, use its corresponding `surface_container*` and
`on_surface` neutral tokens with the same meaning.

Configure Matugen to write its rendered output to:

```text
${XDG_CONFIG_HOME:-$HOME/.config}/clavis-fish-theme/colors.conf
```

Expand that path in the environment where Matugen is configured and set its
TOML `output_path` to the resulting absolute path. For example, a shell
variable can help prepare the value before editing the Matugen config:

```toml
# output_path = "/expanded/path/to/.config/clavis-fish-theme/colors.conf"
```

The theme itself does not run Matugen and does not create the destination.

## Prompt layout

The ordinary `fish_prompt` makes one Rust renderer call and receives:

```text

╭─ [duration] 󰜥 [path] 󰜥 [git]       [language] 󰜥 [time]
╰─  command
```

The leading blank line separates command history/output from the next full
prompt. The left and right module order is fixed. The folded frame and spaced
wave connectors retain the original Clavis visual structure; connectors are
generated only between visible modules. The complete `fish_right_prompt` is
empty in the ordinary state because Fish cannot place a right prompt on the
first line of a multiline prompt.

The second line triangle includes one trailing command-input separator space.
Success uses `arrow`; a failed previous command uses only
`error_foreground` for the transparent triangle. Duration and time retain their
normal capsule colors regardless of status.

Fish 4.1's native final rendering calls the same functions with
`--final-rendering`:

```text
fish_prompt --final-rendering       -> historical `` only
fish_right_prompt --final-rendering -> transient time
```

The transient right time keeps `right_margin`, while the arrow keeps
`left_margin`.

The renderer measures display cells rather than UTF-8 byte length. ANSI CSI
and OSC sequences are ignored, combining graphemes occupy their display width,
and CJK path components are measured as two cells. The first line reserves one
safe cell below `$COLUMNS` to avoid a last-column autowrap followed by an
explicit newline.

When the first line is too narrow, the exact fallback order is:

```text
hide language
hide time
Git branch-only mode
hide Git
shorten path through middle-directory variants
path current directory only
hide path
hide duration last
```

No module is cut in the middle, no ANSI sequence is truncated, and connectors
are never left behind by a hidden module.

## Modules and performance

- Duration uses Fish's `$CMD_DURATION` in milliseconds. It shows every command,
  including `0ms`, after a private `fish_postexec` event has observed the first
  command. Formatting is `ms`, one-decimal seconds, compact minutes, and
  compact hours.
- Path uses the Fish process current directory and `$HOME` (shown as `~`), with
  the original Clavis directory icons, a configurable 40-cell default cap, and
  bounded Unicode-safe shortening. It does not run `pwd`.
- Git first looks for `.git` metadata in ancestors, then makes one
  `git status --porcelain=v2 --branch` call with a 200ms timeout. It never does
  network work, reads remotes, or launches a second Git command for operation
  labels. A timeout/error hides Git.
- Language detection finds the nearest Git/project root and scans it without
  following symlinked directories. It is bounded to eight levels, 2,048
  entries, and 10ms, skips common generated/vendor directories, and displays
  at most the three most-used detected languages. It never starts a
  compiler/interpreter.
- Configuration is two small direct reads. There is no daemon, socket, cache
  service, or per-module Rust process.

The command-line renderer is intentionally small:

```text
clavis-fish-theme init fish
clavis-fish-theme prompt ...       # init's internal renderer interface
clavis-fish-theme --version
clavis-fish-theme --help
```

## Development checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

The repository's tests cover duration boundaries, color precedence and
fallbacks, ANSI/Unicode widths, connector invariants, Git porcelain parsing,
and margin-safe fill behavior. Fish integration and installation checks are
also run during release validation.
