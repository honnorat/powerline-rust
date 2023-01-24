use crate::modules::*;
use crate::Color;

#[derive(Copy, Clone)]
pub struct SimpleTheme;

impl CmdScheme for SimpleTheme {
    const CMD_FAILED_BG: Color = Color(161);
    const CMD_FAILED_FG: Color = Color(15);
    const CMD_PASSED_BG: Color = Color(236);
    const CMD_PASSED_FG: Color = Color(15);
}

impl CwdScheme for SimpleTheme {
    const CWD_FG: Color = Color(254);
    const HOME_BG: Color = Color(25);
    const HOME_FG: Color = Color(15);
    const PATH_BG: Color = Color(237);
    const PATH_FG: Color = Color(250);
    const SEPARATOR_FG: Color = Color(244);
}

impl ExitCodeScheme for SimpleTheme {
    const EXIT_CODE_BG: Color = Color(161);
    const EXIT_CODE_FG: Color = Color(15);
}

impl UserScheme for SimpleTheme {
    const USERNAME_BG: Color = Color(240);
    const USERNAME_FG: Color = Color(250);
    const USERNAME_ROOT_BG: Color = Color(124);
}

impl HostScheme for SimpleTheme {
    const HOSTNAME_BG: Color = Color(235);
    const HOSTNAME_FG: Color = Color(221);
    const SSH_BG: Color = Color(166);   // medium orange
    const SSH_FG: Color = Color(254);   // very-light grey
}

impl ReadOnlyScheme for SimpleTheme {
    const READONLY_BG: Color = Color(124);
    const READONLY_FG: Color = Color(254);
}

#[cfg(feature = "time")]
impl TimeScheme for SimpleTheme {
    const TIME_BG: Color = Color(238);
    const TIME_FG: Color = Color(250);
}

impl GitScheme for SimpleTheme {
    const GIT_AHEAD_BG: Color = Color(240);
    const GIT_AHEAD_FG: Color = Color(250);
    const GIT_BEHIND_BG: Color = Color(240);
    const GIT_BEHIND_FG: Color = Color(250);
    const GIT_CONFLICTED_BG: Color = Color(9);
    const GIT_CONFLICTED_FG: Color = Color(15);
    const GIT_NOTSTAGED_BG: Color = Color(130);
    const GIT_NOTSTAGED_FG: Color = Color(15);
    const GIT_REPO_CLEAN_BG: Color = Color(28);     // dark green
    const GIT_REPO_CLEAN_FG: Color = Color(254);    // very-light grey
    const GIT_REPO_DIRTY_BG: Color = Color(160);    // red
    const GIT_REPO_DIRTY_FG: Color = Color(254);    // very-light grey
    const GIT_STAGED_BG: Color = Color(28);
    const GIT_STAGED_FG: Color = Color(15);
    const GIT_UNTRACKED_BG: Color = Color(52);
    const GIT_UNTRACKED_FG: Color = Color(15);
}

impl VirtualEnvScheme for SimpleTheme {
    const PYVENV_BG: Color = Color(31);
    const PYVENV_FG: Color = Color(254);
}
