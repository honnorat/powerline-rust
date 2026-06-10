use std::path::Path;
use std::process::Command;

use super::GitStats;

/// Parse the leading run of ASCII digits in `s` as a `u32`. Returns 0 if there are none.
fn leading_u32(s: &str) -> u32 {
    let end = s.bytes().take_while(u8::is_ascii_digit).count();
    s[..end].parse().unwrap_or(0)
}

/// Pull the `ahead N`/`behind N` counts out of git's `[ahead 1, behind 2]` suffix.
fn extract_ahead_behind(s: &str) -> (u32, u32) {
    // The `+ 1` skips the space that always follows the keyword.
    let after = |needle: &str| s.find(needle).map(|pos| leading_u32(&s[pos + needle.len() + 1..])).unwrap_or(0);
    (after("ahead"), after("behind"))
}

/// Branch line looks like `## main...origin/main [ahead 1, behind 2]` (tracked)
/// or `## main` (untracked) or `## HEAD (no branch)` (detached).
fn get_branch_name(line: &str) -> Option<&str> {
    let rest = line.get(3..)?;
    if let Some(pos) = rest.find("...") {
        return rest.get(..pos);
    }
    // Untracked local branch: name runs to end-of-line or to a `[...]` block.
    let end = rest.find(' ').unwrap_or(rest.len());
    if end < rest.len() && !rest[end..].trim_start().starts_with('[') {
        return None;
    }
    rest.get(..end)
}

/// Detached HEAD: `git describe` for the closest tag/short hash, prefixed with ⚓.
fn get_detached_branch_name() -> String {
    let output = Command::new("git").args(["describe", "--tags", "--always"]).output();
    match output {
        Ok(out) if out.status.success() => {
            let name = std::str::from_utf8(&out.stdout).unwrap_or("").lines().next().unwrap_or("");
            format!("\u{2693}{}", name)
        },
        _ => "Big Bang".to_owned(),
    }
}

/// Shell out to `git status --porcelain -b` and parse its short output.
pub fn run_git(_: &Path) -> GitStats {
    let Ok(out) = Command::new("git").args(["status", "--porcelain", "-b"]).output() else {
        return GitStats::default();
    };
    let stdout = out.stdout;
    let mut lines = stdout.split(|&b| b == b'\n');
    let branch_line = std::str::from_utf8(lines.next().unwrap_or(b"")).unwrap_or("");

    let (mut ahead, mut behind) = (0, 0);
    let (mut non_staged, mut staged, mut conflicted, mut untracked) = (0, 0, 0, 0);

    let branch_name = match get_branch_name(branch_line) {
        Some(name) => {
            if let Some(pos) = branch_line.find('[') {
                let (a, b) = extract_ahead_behind(&branch_line[pos..]);
                ahead = a;
                behind = b;
            }
            name.to_owned()
        },
        None => get_detached_branch_name(),
    };

    for line in lines {
        let Some(entry) = line.get(..2).and_then(|b| std::str::from_utf8(b).ok()) else { continue };
        match entry {
            "??" => untracked += 1,
            "DD" | "AU" | "UD" | "UA" | "UU" | "DU" | "AA" => conflicted += 1,
            _ => {
                let mut bytes = entry.bytes();
                let a = bytes.next().unwrap_or(b' ');
                let b = bytes.next().unwrap_or(b' ');
                if b != b' ' { non_staged += 1; }
                if a != b' ' { staged += 1; }
            },
        }
    }

    GitStats { untracked, ahead, behind, non_staged, staged, conflicted, branch_name }
}
