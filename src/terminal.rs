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

impl FgColor {
    /// Reinterpret this foreground colour as the same-indexed background.
    pub fn transpose(self) -> BgColor {
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
    pub fn transpose(self) -> FgColor {
        FgColor(self.0)
    }
}

impl From<Color> for BgColor {
    fn from(c: Color) -> Self {
        BgColor(c.0)
    }
}

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

impl std::fmt::Display for BgColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{OPEN}{ESC}48;5;{}m{CLOSE}", self.0)
    }
}

impl std::fmt::Display for FgColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{OPEN}{ESC}38;5;{}m{CLOSE}", self.0)
    }
}

impl std::fmt::Display for Reset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // zsh wraps fg and bg resets separately so each stays width-zero.
        #[cfg(feature = "zsh-shell")]
        return f.write_str("%{\x1b[39m%}%{\x1b[49m%}");
        #[cfg(not(feature = "zsh-shell"))]
        return write!(f, "{OPEN}{ESC}0m{CLOSE}");
    }
}
