use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct ReadOnly<S>(PhantomData<S>);

pub trait ReadOnlyScheme {
    const READONLY_FG: Color;
    const READONLY_BG: Color;
    const READONLY_SYMBOL: &'static str = "";
}

impl<S: ReadOnlyScheme> ReadOnly<S> {
    pub fn new() -> ReadOnly<S> {
        ReadOnly(PhantomData)
    }
}

impl<S: ReadOnlyScheme> Module for ReadOnly<S> {
    /// Render a lock symbol when the current directory is not writable.
    fn append_segments(&mut self, powerline: &mut Powerline) {
        // `c"./"` is a C-string literal; `libc::access` returns 0 when the check passes, so a non-zero result
        // means write access is denied.
        let readonly = unsafe { libc::access(c"./".as_ptr(), libc::W_OK) != 0 };

        if readonly {
            powerline.add_segment(S::READONLY_SYMBOL, Style::simple(S::READONLY_FG, S::READONLY_BG));
        }
    }
}
