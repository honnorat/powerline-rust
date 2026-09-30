use std::marker::PhantomData;
use std::{env, path};

use super::Module;
use crate::{Color, Powerline, Style};

/// Current working directory, split into one segment per path component.
///
/// `PhantomData<S>` makes `S` a *type-only* parameter — it carries no runtime data, but lets
/// every `const Color` in `S` be inlined at compile time.
pub struct Cwd<S> {
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
    pub fn new(max_length: usize, wanted_seg_num: usize, resolve_symlinks: bool) -> Self {
        Self { max_length, wanted_seg_num, resolve_symlinks, scheme: PhantomData }
    }

    /// Labels to display for the absolute path `cwd` (not `/`): the home symbol if `cwd` is under
    /// `home`, then one label per component, the middle ones collapsed into `…` if the path is too
    /// long. Pure function of its inputs, so it can be unit-tested without touching the environment.
    fn labels<'a>(&self, cwd: &'a str, home: Option<&str>) -> Vec<&'a str> {
        let mut labels = Vec::new();
        let mut cwd = cwd;

        // Match on a whole path component: `$HOME=/home/user` must not swallow `/home/username`. A
        // trailing slash in `$HOME` is dropped so the remainder always starts with `/` (or is
        // empty). An empty `$HOME` (or `/`) would match every path, so it is ignored.
        if let Some(home) = home.map(|h| h.trim_end_matches('/')).filter(|h| !h.is_empty())
            && let Some(rest) = cwd.strip_prefix(home)
            && (rest.is_empty() || rest.starts_with('/'))
        {
            labels.push(S::CWD_HOME_SYMBOL);
            cwd = rest;
        }

        let depth = cwd.matches('/').count();

        if cwd.len() > self.max_length && depth > self.wanted_seg_num {
            let left = self.wanted_seg_num / 2;
            let right = self.wanted_seg_num - left;

            labels.extend(cwd.split('/').skip(1).take(left));
            labels.push("\u{2026}");
            labels.extend(cwd.split('/').skip(depth - right + 1));
        } else {
            labels.extend(cwd.split('/').skip(1));
        }
        labels
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

        // `to_string_lossy` returns `Cow::Borrowed` for valid UTF-8 (the common case on Linux), so
        // no allocation happens here. Bind the `Cow` so its borrow lives for the rest of the
        // function.
        let cwd_cow = current_dir.to_string_lossy();
        let cwd: &str = &cwd_cow;

        if cwd == "/" {
            return powerline.add_segment('/', Style::simple(path_fg, path_bg));
        }

        let home = env::var("HOME").ok();
        let segment_style = Style::special(path_fg, path_bg, '\u{E0B1}', S::SEPARATOR_FG);
        for label in self.labels(cwd, home.as_deref()) {
            powerline.add_segment(label, segment_style);
        }

        // Upgrade the trailing thin divider to a solid powerline separator, so the boundary against the next
        // module is rendered normally.
        if let Some(style) = powerline.last_style_mut() {
            style.sep = '\u{E0B0}';
            style.sep_fg = style.bg.transpose();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::SimpleTheme;

    /// `max_length = 45`, keep 4 components when collapsing — the settings of `src/bin/powerline.rs`.
    fn cwd() -> Cwd<SimpleTheme> {
        Cwd::new(45, 4, false)
    }

    #[test]
    fn outside_home() {
        assert_eq!(cwd().labels("/usr/local/bin", Some("/home/user")), ["usr", "local", "bin"]);
        assert_eq!(cwd().labels("/usr/local/bin", None), ["usr", "local", "bin"]);
    }

    #[test]
    fn home_and_below() {
        assert_eq!(cwd().labels("/home/user", Some("/home/user")), ["~"]);
        assert_eq!(cwd().labels("/home/user/code", Some("/home/user")), ["~", "code"]);
        assert_eq!(cwd().labels("/home/user/code", Some("/home/user/")), ["~", "code"]);
    }

    #[test]
    fn home_is_a_whole_component() {
        assert_eq!(cwd().labels("/home/username/code", Some("/home/user")), ["home", "username", "code"]);
    }

    #[test]
    fn empty_or_root_home_is_ignored() {
        assert_eq!(cwd().labels("/etc", Some("")), ["etc"]);
        assert_eq!(cwd().labels("/etc", Some("/")), ["etc"]);
    }

    #[test]
    fn long_path_is_collapsed() {
        let short = Cwd::<SimpleTheme>::new(5, 4, false);
        assert_eq!(short.labels("/a/b/c/d/e/f/g", None), ["a", "b", "\u{2026}", "f", "g"]);
        assert_eq!(short.labels("/home/user/a/b/c/d/e", Some("/home/user")), ["~", "a", "b", "\u{2026}", "d", "e"]);
    }

    #[test]
    fn long_but_shallow_path_is_kept() {
        let short = Cwd::<SimpleTheme>::new(5, 4, false);
        assert_eq!(short.labels("/aaaaaa/bbbbbb", None), ["aaaaaa", "bbbbbb"]);
    }
}
