use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style, utils};

pub struct Host<S> {
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
    pub fn new() -> Self {
        Self { show_on_local: true, scheme: PhantomData }
    }

    /// Render only when the shell is detected as remote (SSH).
    pub fn show_on_remote_shell() -> Self {
        Self { show_on_local: false, scheme: PhantomData }
    }
}

impl<S: HostScheme> Default for Host<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Read the kernel hostname and return only the part before the first dot.
fn short_hostname() -> Option<String> {
    let mut buf = [0u8; 256];
    // SAFETY: `buf` is valid for writes of `buf.len() - 1` bytes. The last byte is never written and
    // stays 0, so the result is NUL-terminated even if POSIX truncation leaves it unterminated.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len() - 1) };
    if rc != 0 {
        return None;
    }
    // Bail on any conversion failure.
    let s = std::ffi::CStr::from_bytes_until_nul(&buf).ok()?.to_str().ok()?;
    Some(s.split('.').next()?.to_owned())
}

impl<S: HostScheme> Module for Host<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        let is_remote = utils::is_remote_shell();
        if (self.show_on_local || is_remote)
            && let Some(host) = short_hostname()
        {
            if is_remote {
                powerline.add_short_segment("  ", Style::simple(S::SSH_FG, S::SSH_BG));
            }
            powerline.add_segment(host, Style::nosep(S::HOSTNAME_FG, S::HOSTNAME_BG));
        }
    }
}
