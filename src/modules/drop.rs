use std::env;
use std::marker::PhantomData;

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
    /// Render the active `drop run` environment name, in bold. See https://droprun.sh/
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let Ok(dropenv) = env::var("DROP_ENV") else { return };
        let trimmed = dropenv.trim();
        if !trimmed.is_empty() {
            powerline.add_segment(trimmed, Style::simple(S::DROPENV_FG, S::DROPENV_BG).bold())
        }
    }
}
