use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style, utils};

pub struct User<S> {
    show_on_local: bool,
    scheme: PhantomData<S>,
}

pub trait UserScheme {
    const USERNAME_ROOT_BG: Color;
    const USERNAME_BG: Color;
    const USERNAME_FG: Color;
}

impl<S: UserScheme> User<S> {
    /// Always render the username.
    pub fn new() -> Self {
        Self { show_on_local: true, scheme: PhantomData }
    }

    /// Render only when the shell is detected as remote (SSH).
    pub fn show_on_remote_shell() -> Self {
        Self { show_on_local: false, scheme: PhantomData }
    }
}

impl<S: UserScheme> Default for User<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: UserScheme> Module for User<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        // No passwd entry for the uid (containers, LDAP/SSSD down): skip the segment rather than
        // panic, which would blank the whole prompt.
        if (self.show_on_local || utils::is_remote_shell())
            && let Some(user) = uzers::get_user_by_uid(uzers::get_current_uid())
        {
            let bg = if user.uid() == 0 { S::USERNAME_ROOT_BG } else { S::USERNAME_BG };

            powerline
                .add_short_segment(format!("{} ", user.name().to_string_lossy()), Style::simple(S::USERNAME_FG, bg));
        }
    }
}
