use std::fmt;

/// A 256-colour palette index (xterm-256 / ANSI).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8);

/// A `Color` tagged as a background; `Display` writes an ANSI `48;5;N` escape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BgColor(u8);

/// A `Color` tagged as a foreground; `Display` writes an ANSI `38;5;N` escape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FgColor(u8);

/// `Display`s the SGR reset sequence (fg + bg back to default).
pub struct Reset;

/// `Display`s the SGR bold-on (`1`) or bold-off (`22`) sequence.
pub struct Bold(pub bool);

impl FgColor {
    /// Reinterpret this foreground colour as the same-indexed background.
    pub fn into_bg(self) -> BgColor {
        BgColor(self.0)
    }
}

impl From<Color> for FgColor {
    fn from(c: Color) -> Self {
        FgColor(c.0)
    }
}

impl BgColor {
    /// Reinterpret this background colour as the same-indexed foreground.
    pub fn into_fg(self) -> FgColor {
        FgColor(self.0)
    }
}

impl From<Color> for BgColor {
    fn from(c: Color) -> Self {
        BgColor(c.0)
    }
}

// Cargo assumes features are additive, but exactly one shell feature must be enabled here.
#[cfg(not(any(feature = "bash-shell", feature = "bare-shell", feature = "zsh-shell")))]
compile_error!("enable one shell feature: `bash-shell`, `zsh-shell` or `bare-shell`");

#[cfg(any(
    all(feature = "bash-shell", feature = "bare-shell"),
    all(feature = "bash-shell", feature = "zsh-shell"),
    all(feature = "bare-shell", feature = "zsh-shell"),
))]
compile_error!(
    "shell features are mutually exclusive; `bash-shell` is a default feature, so build with e.g. \
     `--no-default-features --features=zsh-shell,gitoxide`"
);

// Per-shell wrappers around an ANSI escape sequence. OPEN/CLOSE bracket the
// escape so the shell doesn't count it as visible width; ESC is the CSI introducer.
#[cfg(feature = "bash-shell")]
const OPEN: &str = r"\[";
#[cfg(feature = "bash-shell")]
const ESC: &str = r"\e[";
#[cfg(feature = "bash-shell")]
const CLOSE: &str = r"\]";

#[cfg(feature = "bare-shell")]
const OPEN: &str = "";
#[cfg(feature = "bare-shell")]
const ESC: &str = "\x1b[";
#[cfg(feature = "bare-shell")]
const CLOSE: &str = "";

#[cfg(feature = "zsh-shell")]
const OPEN: &str = "%{";
#[cfg(feature = "zsh-shell")]
const ESC: &str = "\x1b[";
#[cfg(feature = "zsh-shell")]
const CLOSE: &str = "%}";

impl fmt::Display for BgColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{OPEN}{ESC}48;5;{}m{CLOSE}", self.0)
    }
}

impl fmt::Display for FgColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{OPEN}{ESC}38;5;{}m{CLOSE}", self.0)
    }
}

impl fmt::Display for Bold {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = if self.0 { 1 } else { 22 };
        write!(f, "{OPEN}{ESC}{code}m{CLOSE}")
    }
}

impl fmt::Display for Reset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // zsh wraps fg and bg resets separately so each stays width-zero.
        if cfg!(feature = "zsh-shell") {
            f.write_str("%{\x1b[39m%}%{\x1b[49m%}")
        } else {
            write!(f, "{OPEN}{ESC}0m{CLOSE}")
        }
    }
}
