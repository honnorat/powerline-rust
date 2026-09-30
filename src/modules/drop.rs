use std::env;
use std::marker::PhantomData;
use std::path::Path;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct DropEnv<S: DropEnvScheme> {
    scheme: PhantomData<S>,
}

pub trait DropEnvScheme {
    const DROPENV_FG: Color;
    const DROPENV_BG: Color;
}

impl<S: DropEnvScheme> DropEnv<S> {
    pub fn new() -> DropEnv<S> {
        DropEnv { scheme: PhantomData }
    }
}

impl<S: DropEnvScheme> Module for DropEnv<S> {
    /// Render the active venv (Python venv, uv, or conda) as `[name]`.
    fn append_segments(&mut self, powerline: &mut Powerline) {
        // DROP_ENV is set by `drop run`. See https://droprun.sh/
        let dropenv = ["DROP_ENV"]
            .iter()
            .find_map(|k| env::var(k).ok());
        // Fall back to the raw string in those cases; skip the segment only if both end up empty.
        let Some(dropenv_path) = dropenv else { return };
        let raw = Path::new(&dropenv_path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(dropenv_path);
        let env_name = raw.replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
        let trimmed = env_name.trim();
        if !trimmed.is_empty() {
            powerline.add_segment(format!("{}", trimmed), Style::simple(S::DROPENV_FG, S::DROPENV_BG).bold())
        }
    }
}
