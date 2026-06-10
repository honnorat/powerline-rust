use std::marker::PhantomData;
use std::{env, path};

use super::Module;
use crate::{Color, Powerline, Style};

/// Current working directory, split into one segment per path component.
///
/// `PhantomData<S>` makes `S` a *type-only* parameter — it carries no runtime data, but lets
/// every `const Color` in `S` be inlined at compile time.
pub struct Cwd<S: CwdScheme> {
    max_length: usize,
    wanted_seg_num: usize,
    resolve_symlinks: bool,
    scheme: PhantomData<S>,
}

pub trait CwdScheme {
    const CWD_FG: Color;
    const PATH_FG: Color;
    const PATH_BG: Color;
    const HOME_FG: Color;
    const HOME_BG: Color;
    const SEPARATOR_FG: Color;
    const CWD_MISSING_FG: Color;
    const CWD_MISSING_BG: Color;
    const CWD_HOME_SYMBOL: &'static str = "~";
}

impl<S: CwdScheme> Cwd<S> {
    /// - `max_length`: collapse the path with an ellipsis if it would be longer.
    /// - `wanted_seg_num`: how many components to keep when collapsing.
    /// - `resolve_symlinks`: if true prefer the canonical path, else honour `$PWD`.
    pub fn new(max_length: usize, wanted_seg_num: usize, resolve_symlinks: bool) -> Cwd<S> {
        Cwd { max_length, wanted_seg_num, resolve_symlinks, scheme: PhantomData }
    }
}

impl<S: CwdScheme> Module for Cwd<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let current_dir = if self.resolve_symlinks {
            env::current_dir().ok().or_else(|| env::var("PWD").ok().map(path::PathBuf::from))
        } else {
            env::var("PWD").ok().map(path::PathBuf::from).or_else(|| env::current_dir().ok())
        };

        // Match-with-guard: first arm matches Some only if `dir.exists()` is true.
        let (current_dir, path_fg, path_bg) = match current_dir {
            Some(dir) if dir.exists() => (dir, S::PATH_FG, S::PATH_BG),
            Some(dir) => (dir, S::CWD_MISSING_FG, S::CWD_MISSING_BG),
            None => return,
        };

        // `to_string_lossy` returns `Cow::Borrowed` for valid UTF-8 (the common case on Linux), so no
        // allocation happens here. Bind the `Cow` so its borrow lives for the rest of the function.
        let cwd_cow = current_dir.to_string_lossy();
        let mut cwd: &str = &cwd_cow;

        if cwd == "/" {
            return powerline.add_segment('/', Style::simple(path_fg, path_bg));
        }

        let segment_style = Style::special(path_fg, path_bg, '\u{E0B1}', S::SEPARATOR_FG);
        let push = |powerline: &mut Powerline, val: &str| {
            powerline.add_segment(val, segment_style.clone());
        };

        if let Ok(home_str) = env::var("HOME") {
            if let Some(rest) = cwd.strip_prefix(home_str.as_str()) {
                push(powerline, S::CWD_HOME_SYMBOL);
                cwd = rest;
            }
        }

        let depth = cwd.matches('/').count();

        if (cwd.len() > self.max_length) && (depth > self.wanted_seg_num) {
            let left = self.wanted_seg_num / 2;
            let right = self.wanted_seg_num - left;

            for val in cwd.split('/').skip(1).take(left) {
                push(powerline, val);
            }
            push(powerline, "\u{2026}");
            for val in cwd.split('/').skip(depth - right + 1) {
                push(powerline, val);
            }
        } else {
            for val in cwd.split('/').skip(1) {
                push(powerline, val);
            }
        };

        // Upgrade the trailing thin divider to a solid powerline separator, so the boundary against the next
        // module is rendered normally.
        if let Some(style) = powerline.last_style_mut() {
            style.sep = '\u{E0B0}';
            style.sep_fg = style.bg.transpose();
        }
    }
}
