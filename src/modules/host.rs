use std::marker::PhantomData;

use super::Module;
use crate::{utils, Color, Powerline, Style};

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

impl<S: HostScheme> Module for Host<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        if self.show_on_local || utils::is_remote_shell() {
            if let Ok(host) = hostname::get() {
                if utils::is_remote_shell() {
                    powerline.add_short_segment("\u{108AA}\u{00A0}", Style::simple(S::SSH_FG, S::SSH_BG));
                }
                powerline.add_segment(host.to_str().unwrap(), Style::nosep(S::HOSTNAME_FG, S::HOSTNAME_BG));
            }
        }
    }
}
