use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style, utils};

pub struct Host<S: HostScheme> {
    show_on_local: bool,
    scheme: PhantomData<S>,
}

pub trait HostScheme {
    const HOSTNAME_FG: Color;
    const HOSTNAME_BG: Color;
    const SSH_FG: Color;
    const SSH_BG: Color;
}

impl<S: HostScheme> Host<S> {
    /// Always render the hostname.
    pub fn new() -> Host<S> {
        Host { show_on_local: true, scheme: PhantomData }
    }

    /// Render only when the shell is detected as remote (SSH).
    pub fn show_on_remote_shell() -> Host<S> {
        Host { show_on_local: false, scheme: PhantomData }
    }
}

/// Read the kernel hostname and return only the part before the first dot.
fn short_hostname() -> Option<String> {
    let mut buf = [0u8; 256];
    // `unsafe` is required to call C: we promise the pointer + length are valid.
    // Pass len-1 so any truncated result stays NUL-terminated.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len() - 1) };
    if rc != 0 {
        return None;
    }
    // `?` here propagates `None` from `Option` — bail on any conversion failure.
    let s = std::ffi::CStr::from_bytes_until_nul(&buf).ok()?.to_str().ok()?;
    Some(s.split('.').next()?.to_owned())
}

impl<S: HostScheme> Module for Host<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let is_remote = utils::is_remote_shell();
        if self.show_on_local || is_remote {
            if let Some(host) = short_hostname() {
                if is_remote {
                    powerline.add_short_segment(" \u{eb3a} ", Style::simple(S::SSH_FG, S::SSH_BG));
                }
                powerline.add_segment(host, Style::nosep(S::HOSTNAME_FG, S::HOSTNAME_BG));
            }
        }
    }
}
