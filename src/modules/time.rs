use std::ffi::CString;
use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct Time<S: TimeScheme> {
    time_format: CString,
    scheme: PhantomData<S>,
}

pub trait TimeScheme {
    const TIME_BG: Color;
    const TIME_FG: Color;
}

impl<S: TimeScheme> Time<S> {
    pub fn new() -> Time<S> {
        Self::with_time_format("%H:%M:%S")
    }

    pub fn with_time_format(time_format: &str) -> Time<S> {
        Time {
            time_format: CString::new(time_format).expect("time format contains NUL byte"),
            scheme: PhantomData,
        }
    }
}

impl<S: TimeScheme> Module for Time<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let mut buf = [0u8; 64];
        let written = unsafe {
            let now = libc::time(std::ptr::null_mut());
            let mut tm: libc::tm = std::mem::zeroed();
            if libc::localtime_r(&now, &mut tm).is_null() {
                return;
            }
            libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                self.time_format.as_ptr(),
                &tm,
            )
        };
        if written == 0 {
            return;
        }
        if let Ok(s) = std::str::from_utf8(&buf[..written]) {
            powerline.add_segment(s, Style::simple(S::TIME_FG, S::TIME_BG));
        }
    }
}
