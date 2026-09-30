use std::ffi::CString;
use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct Time<S> {
    time_format: CString,
    scheme: PhantomData<S>,
}

pub trait TimeScheme {
    const TIME_BG: Color;
    const TIME_FG: Color;
}

impl<S: TimeScheme> Time<S> {
    /// Default to `HH:MM:SS`.
    pub fn new() -> Self {
        Self::with_time_format("%H:%M:%S")
    }

    /// Custom `strftime(3)` format string.
    pub fn with_time_format(time_format: &str) -> Self {
        Self {
            // `CString::new` fails only on interior NUL bytes — caller error.
            time_format: CString::new(time_format).expect("time format contains NUL byte"),
            scheme: PhantomData,
        }
    }
}

impl<S: TimeScheme> Default for Time<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: TimeScheme> Module for Time<S> {
    /// Format the current local time via libc and render it as one segment.
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let mut buf = [0u8; 64];
        // SAFETY: `time` accepts a null pointer. A zeroed `tm` is a valid value (its only pointer
        // field, `tm_zone`, is null) and `localtime_r` overwrites it. `buf` is valid for `buf.len()`
        // bytes, which `strftime` never exceeds.
        let written = unsafe {
            let now = libc::time(std::ptr::null_mut());
            let mut tm: libc::tm = std::mem::zeroed();
            if libc::localtime_r(&now, &mut tm).is_null() {
                return;
            }
            libc::strftime(buf.as_mut_ptr().cast(), buf.len(), self.time_format.as_ptr(), &tm)
        };
        if written == 0 {
            return;
        }
        if let Ok(s) = std::str::from_utf8(&buf[..written]) {
            powerline.add_segment(s, Style::simple(S::TIME_FG, S::TIME_BG));
        }
    }
}
