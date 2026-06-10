use std::env;
use std::marker::PhantomData;
use std::path::Path;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct VirtualEnv<S: VirtualEnvScheme> {
    scheme: PhantomData<S>,
}

pub trait VirtualEnvScheme {
    const PYVENV_FG: Color;
    const PYVENV_BG: Color;
}

impl<S: VirtualEnvScheme> VirtualEnv<S> {
    pub fn new() -> VirtualEnv<S> {
        VirtualEnv { scheme: PhantomData }
    }
}

impl<S: VirtualEnvScheme> Module for VirtualEnv<S> {
    /// Render the active venv (Python venv, uv, or conda) as `[name]`.
    fn append_segments(&mut self, powerline: &mut Powerline) {
        // VIRTUAL_ENV_PROMPT is set by `uv venv --prompt`.
        let venv = ["VIRTUAL_ENV_PROMPT", "VIRTUAL_ENV", "CONDA_ENV_PATH", "CONDA_DEFAULT_ENV"]
            .iter()
            .find_map(|k| env::var(k).ok());
        // `file_name()` is `None` for "", "/", ".", or a path ending in "..". `VIRTUAL_ENV_PROMPT` in
        // particular is a free-form label (e.g. `uv venv --prompt ""`), so we cannot assume it is a path.
        // Fall back to the raw string in those cases; skip the segment only if both end up empty.
        let Some(venv_path) = venv else { return };
        let raw = Path::new(&venv_path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(venv_path);
        let venv_name = raw.replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
        let trimmed = venv_name.trim();
        if !trimmed.is_empty() {
            powerline.add_segment(format!("[{}]", trimmed), Style::simple(S::PYVENV_FG, S::PYVENV_BG))
        }
    }
}
