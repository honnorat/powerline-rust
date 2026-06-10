use std::env;
use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct Cmd<S: CmdScheme> {
    status: Option<bool>,
    scheme: PhantomData<S>,
}

pub trait CmdScheme {
    const CMD_PASSED_FG: Color;
    const CMD_PASSED_BG: Color;
    const CMD_FAILED_BG: Color;
    const CMD_FAILED_FG: Color;
    const CMD_ROOT_SYMBOL: &'static str = "#";
    const CMD_USER_SYMBOL: &'static str = "$";
}

impl<S: CmdScheme> Cmd<S> {
    /// Derive pass/fail from `argv[1]` (the previous command's exit code).
    pub fn new() -> Cmd<S> {
        Cmd { status: None, scheme: PhantomData }
    }

    /// Force the pass/fail state instead of reading the exit code.
    pub fn with_status(status: bool) -> Cmd<S> {
        Cmd { status: Some(status), scheme: PhantomData }
    }
}

impl<S: CmdScheme> Module for Cmd<S> {
    /// Render the prompt symbol coloured by command success and by uid (root vs user).
    fn append_segments(&mut self, powerline: &mut Powerline) {
        // `unwrap_or_else` evaluates the closure only when `self.status` is `None`.
        let passed = self.status.unwrap_or_else(|| env::args().nth(1).as_deref() == Some("0"));
        let (fg, bg) = if passed {
            (S::CMD_PASSED_FG, S::CMD_PASSED_BG)
        } else {
            (S::CMD_FAILED_FG, S::CMD_FAILED_BG)
        };

        let special = if uzers::get_current_uid() == 0 { S::CMD_ROOT_SYMBOL } else { S::CMD_USER_SYMBOL };
        powerline.add_segment(special, Style::simple(fg, bg));
    }
}
