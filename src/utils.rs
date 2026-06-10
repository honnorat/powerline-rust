use std::env;

/// True if any standard SSH-related env var is set.
pub fn is_remote_shell() -> bool {
    env::var_os("SSH_CLIENT").is_some() || env::var_os("SSH_TTY").is_some() || env::var_os("SSH_CONNECTION").is_some()
}
