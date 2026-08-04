use crate::config::ColorPair;
use crate::segment::{capsule, Segment};
use crate::width::truncate_visible;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const GIT_TIMEOUT: Duration = Duration::from_millis(200);

#[derive(Default)]
struct GitStatus {
    branch: String,
    oid: String,
    detached: bool,
    ahead: usize,
    behind: usize,
    staged: usize,
    modified: usize,
    deleted: usize,
    renamed: usize,
    untracked: usize,
    conflicted: usize,
}

pub fn render(colors: ColorPair) -> Vec<Segment> {
    let current = env::current_dir().ok();
    let Some(current) = current else {
        return Vec::new();
    };
    let Some(git_dir) = find_git_dir(&current) else {
        return Vec::new();
    };
    let Some(output) = run_status(&current) else {
        return Vec::new();
    };
    let status = parse_status(&output);
    let branch = branch_name(&status);
    if branch.is_empty() {
        return Vec::new();
    }

    let mut full = format!("󰘬 {branch}");
    if let Some(operation) = operation(&git_dir) {
        full.push(' ');
        full.push_str(operation);
    }
    if let Some(sync) = sync_text(&status) {
        full.push(' ');
        full.push_str(&sync);
    }
    for change in changes(&status) {
        full.push(' ');
        full.push_str(&change);
    }

    let compact = format!("󰘬 {branch}");
    let full_segment = capsule(colors, &full);
    let compact_segment = capsule(colors, &compact);
    if full_segment == compact_segment {
        vec![full_segment]
    } else {
        vec![full_segment, compact_segment]
    }
}

fn find_git_dir(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        let marker = directory.join(".git");
        if marker.is_dir() {
            return Some(marker);
        }
        if marker.is_file() {
            let content = fs::read_to_string(&marker).ok()?;
            let value = content.trim().strip_prefix("gitdir:")?.trim();
            let path = PathBuf::from(value);
            return Some(if path.is_absolute() {
                path
            } else {
                directory.join(path)
            });
        }
    }
    None
}

fn run_status(directory: &Path) -> Option<String> {
    let mut child = Command::new("git")
        .args([
            "-c",
            "core.quotepath=false",
            "status",
            "--porcelain=v2",
            "--branch",
        ])
        .current_dir(directory)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() >= GIT_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => thread::sleep(Duration::from_millis(2)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_status(output: &str) -> GitStatus {
    let mut status = GitStatus::default();
    for line in output.lines() {
        if let Some(value) = line.strip_prefix("# branch.head ") {
            status.branch = value.trim().to_owned();
            status.detached = status.branch == "(detached)";
        } else if let Some(value) = line.strip_prefix("# branch.oid ") {
            status.oid = value.trim().to_owned();
        } else if let Some(value) = line.strip_prefix("# branch.ab ") {
            let mut counts = value.split_whitespace();
            status.ahead = signed_count(counts.next());
            status.behind = signed_count(counts.next());
        } else if line.starts_with("1 ") {
            count_regular_change(&mut status, line, false);
        } else if line.starts_with("2 ") {
            count_regular_change(&mut status, line, true);
        } else if line.starts_with("u ") {
            status.conflicted += 1;
        } else if line.starts_with("? ") {
            status.untracked += 1;
        }
    }
    status
}

fn count_regular_change(status: &mut GitStatus, line: &str, renamed: bool) {
    let bytes = line.as_bytes();
    if bytes.len() < 4 {
        return;
    }
    let staged = bytes[2] as char;
    let worktree = bytes[3] as char;
    if staged != '.' && staged != ' ' {
        status.staged += 1;
    }
    if matches!(worktree, 'M' | 'T') {
        status.modified += 1;
    }
    if staged == 'D' || worktree == 'D' {
        status.deleted += 1;
    }
    if renamed {
        status.renamed += 1;
    }
}

fn signed_count(value: Option<&str>) -> usize {
    value
        .unwrap_or_default()
        .trim_start_matches(['+', '-'])
        .parse()
        .unwrap_or(0)
}

fn branch_name(status: &GitStatus) -> String {
    let branch = if status.detached {
        let oid = status.oid.trim();
        if oid.is_empty() || oid == "(initial)" {
            "detached".to_owned()
        } else {
            format!("detached@{}", &oid[..oid.len().min(7)])
        }
    } else if status.branch.is_empty()
        || matches!(status.branch.as_str(), "(initial)" | "(unknown)")
    {
        "Git".to_owned()
    } else {
        status.branch.clone()
    };
    truncate_visible(&branch, 16)
}

fn sync_text(status: &GitStatus) -> Option<String> {
    match (status.ahead, status.behind) {
        (0, 0) => None,
        (ahead, behind) if ahead > 0 && behind > 0 => Some(format!("⇕{ahead}/{behind}")),
        (ahead, 0) if ahead > 0 => Some(format!("⇡{ahead}")),
        (0, behind) if behind > 0 => Some(format!("⇣{behind}")),
        _ => None,
    }
}

fn changes(status: &GitStatus) -> Vec<String> {
    let mut result = Vec::new();
    if status.conflicted > 0 {
        result.push(format!("!{}", status.conflicted));
    }
    if status.staged > 0 {
        result.push(format!("+{}", status.staged));
    }
    if status.modified > 0 {
        result.push(format!("~{}", status.modified));
    }
    if status.deleted > 0 {
        result.push(format!("-{}", status.deleted));
    }
    if status.renamed > 0 {
        result.push(format!("»{}", status.renamed));
    }
    if status.untracked > 0 {
        result.push(format!("?{}", status.untracked));
    }
    result.truncate(4);
    result
}

fn operation(git_dir: &Path) -> Option<&'static str> {
    if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
        Some("REBASE")
    } else if git_dir.join("MERGE_HEAD").exists() {
        Some("MERGE")
    } else if git_dir.join("CHERRY_PICK_HEAD").exists() {
        Some("PICK")
    } else if git_dir.join("REVERT_HEAD").exists() {
        Some("REVERT")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{changes, parse_status, sync_text};

    #[test]
    fn parses_porcelain_v2_without_network_work() {
        let status = parse_status(
            "# branch.head main\n# branch.oid abcdef012345\n# branch.ab +2 -1\n1 .M N... 100644 100644 100644 a b\n2 R. N... 100644 100644 100644 a b R100\nu UU N... 100644 100644 100644 100644 a b c\n? new.txt\n",
        );
        assert_eq!(status.branch, "main");
        assert_eq!(status.ahead, 2);
        assert_eq!(status.behind, 1);
        assert_eq!(status.modified, 1);
        assert_eq!(status.renamed, 1);
        assert_eq!(status.conflicted, 1);
        assert_eq!(status.untracked, 1);
        assert_eq!(sync_text(&status).as_deref(), Some("⇕2/1"));
        assert_eq!(changes(&status), vec!["!1", "+1", "~1", "»1"]);
    }
}
