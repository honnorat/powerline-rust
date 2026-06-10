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
    pub fn new() -> Host<S> {
        Host { show_on_local: true, scheme: PhantomData }
    }

    pub fn show_on_remote_shell() -> Host<S> {
        Host { show_on_local: false, scheme: PhantomData }
    }
}

fn short_hostname() -> Option<String> {
    let mut buf = [0u8; 256];
    // Pass len-1 so any truncated result stays NUL-terminated.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len() - 1) };
    if rc != 0 {
        return None;
    }
    let end = buf.iter().position(|&b| b == 0 || b == b'.').unwrap_or(buf.len());
    std::str::from_utf8(&buf[..end]).ok().map(str::to_owned)
}

impl<S: HostScheme> Module for Host<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        if self.show_on_local || utils::is_remote_shell() {
            if let Some(host) = short_hostname() {
                if utils::is_remote_shell() {
                    powerline.add_short_segment(" \u{eb3a} ", Style::simple(S::SSH_FG, S::SSH_BG));
                }
                powerline.add_segment(host, Style::nosep(S::HOSTNAME_FG, S::HOSTNAME_BG));
            }
        }
    }
}
