use std::env;
use std::marker::PhantomData;
use std::path::PathBuf;

use super::Module;
use crate::{Color, Powerline, Style};

/// Branch name shown when the repository has no readable HEAD (e.g. no commit yet).
const UNBORN_BRANCH: &str = "Big Bang";

// Pick the git backend at compile time: link `gix` (default) or shell out to `git`.
//
// Both submodules expose `run_git(&Path) -> GitStats`; aliasing one as `internal` keeps the call site below
// backend-agnostic.
#[cfg(not(feature = "gitoxide"))]
mod process;

#[cfg(not(feature = "gitoxide"))]
use process as internal;

#[cfg(feature = "gitoxide")]
mod gitoxide;

#[cfg(feature = "gitoxide")]
use gitoxide as internal;

pub struct Git<S> {
    scheme: PhantomData<S>,
}

pub trait GitScheme {
    const GIT_AHEAD_BG: Color;
    const GIT_AHEAD_FG: Color;
    const GIT_BEHIND_BG: Color;
    const GIT_BEHIND_FG: Color;
    const GIT_STAGED_BG: Color;
    const GIT_STAGED_FG: Color;
    const GIT_NOTSTAGED_BG: Color;
    const GIT_NOTSTAGED_FG: Color;
    const GIT_UNTRACKED_BG: Color;
    const GIT_UNTRACKED_FG: Color;
    const GIT_CONFLICTED_BG: Color;
    const GIT_CONFLICTED_FG: Color;
    const GIT_REPO_CLEAN_BG: Color;
    const GIT_REPO_CLEAN_FG: Color;
    const GIT_REPO_DIRTY_BG: Color;
    const GIT_REPO_DIRTY_FG: Color;
}

impl<S: GitScheme> Git<S> {
    pub fn new() -> Self {
        Self { scheme: PhantomData }
    }
}

impl<S: GitScheme> Default for Git<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated working-tree state, filled by the active backend.
pub struct GitStats {
    pub untracked: u32,
    pub conflicted: u32,
    pub non_staged: u32,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub branch_name: String,
}

impl GitStats {
    /// True when *any* working-tree change exists (drives the branch colour).
    pub fn is_dirty(&self) -> bool {
        self.untracked > 0 || self.conflicted > 0 || self.staged > 0 || self.non_staged > 0
    }
}

impl Default for GitStats {
    /// Fallback used when the repo cannot be read (e.g. empty repo with no HEAD).
    fn default() -> Self {
        Self {
            untracked: 0,
            conflicted: 0,
            non_staged: 0,
            staged: 0,
            ahead: 0,
            behind: 0,
            branch_name: UNBORN_BRANCH.into(),
        }
    }
}

/// Walk ancestors of the current directory until one contains a `.git` entry.
fn find_git_dir() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    cwd.ancestors().find(|p| p.join(".git").exists()).map(std::path::Path::to_path_buf)
}

impl<S: GitScheme> Module for Git<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let Some(git_dir) = find_git_dir() else { return };

        let stats = internal::run_git(&git_dir);

        let (branch_fg, branch_bg) = if stats.is_dirty() {
            (S::GIT_REPO_DIRTY_FG, S::GIT_REPO_DIRTY_BG)
        } else {
            (S::GIT_REPO_CLEAN_FG, S::GIT_REPO_CLEAN_BG)
        };

        powerline.add_segment(format!(" {}", stats.branch_name), Style::simple(branch_fg, branch_bg));

        let mut add_elem = |count: u32, symbol, fg, bg| match count {
            0 => {},
            1 => powerline.add_segment(symbol, Style::simple(fg, bg)),
            n => powerline.add_segment(format!("{n}{symbol}"), Style::simple(fg, bg)),
        };
        add_elem(stats.ahead, '⬆', S::GIT_AHEAD_FG, S::GIT_AHEAD_BG);
        add_elem(stats.behind, '⬇', S::GIT_BEHIND_FG, S::GIT_BEHIND_BG);
        add_elem(stats.staged, '✔', S::GIT_STAGED_FG, S::GIT_STAGED_BG);
        add_elem(stats.non_staged, '✎', S::GIT_NOTSTAGED_FG, S::GIT_NOTSTAGED_BG);
        add_elem(stats.untracked, '?', S::GIT_UNTRACKED_FG, S::GIT_UNTRACKED_BG);
        add_elem(stats.conflicted, '✼', S::GIT_CONFLICTED_FG, S::GIT_CONFLICTED_BG);
    }
}
