use crate::powerline::Powerline;

mod cmd;
mod cwd;
mod exit_code;
mod git;
mod host;
mod jobs;
mod readonly;
mod user;
mod venv;

#[cfg(feature = "time")]
mod time;

pub use cmd::{Cmd, CmdScheme};
pub use cwd::{Cwd, CwdScheme};
pub use exit_code::{ExitCode, ExitCodeScheme};
pub use git::{Git, GitScheme};
pub use host::{Host, HostScheme};
pub use jobs::{Jobs, JobsScheme};
pub use readonly::{ReadOnly, ReadOnlyScheme};
#[cfg(feature = "time")]
pub use time::{Time, TimeScheme};
pub use user::{User, UserScheme};
pub use venv::{VirtualEnv, VirtualEnvScheme};

pub trait Module {
    fn append_segments(&mut self, powerline: &mut Powerline);
}
