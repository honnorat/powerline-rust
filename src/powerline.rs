use std::fmt::{self, Display, Write};

use crate::modules::Module;
use crate::terminal::*;

/// Foreground/background colours plus the separator glyph emitted *after* this segment.
#[derive(Clone)]
pub struct Style {
    pub fg: FgColor,
    pub bg: BgColor,
    pub sep: char,
    pub sep_fg: FgColor,
}

impl Style {
    /// Solid powerline separator (U+E0B0), separator colour = own background.
    pub fn simple(fg: Color, bg: Color) -> Style {
        Style { fg: fg.into(), bg: bg.into(), sep: '\u{E0B0}', sep_fg: bg.into() }
    }

    /// No separator glyph — a space sits between this segment and the next.
    pub fn nosep(fg: Color, bg: Color) -> Style {
        Style { fg: fg.into(), bg: bg.into(), sep: ' ', sep_fg: bg.into() }
    }

    /// Custom separator glyph and colour (used e.g. for the thin CWD divider).
    pub fn special(fg: Color, bg: Color, sep: char, sep_fg: Color) -> Style {
        Style { fg: fg.into(), bg: bg.into(), sep, sep_fg: sep_fg.into() }
    }
}

/// Accumulating prompt buffer. Segments are appended left-to-right; the separator between segments is emitted
/// lazily when the *next* segment arrives (we need its background colour first).
pub struct Powerline {
    buffer: String,
    last_style: Option<Style>,
}

impl Powerline {
    pub fn new() -> Powerline {
        Powerline { buffer: String::with_capacity(512), last_style: None }
    }

    /// Emit the previous segment's separator (now that we know the new bg), then the new segment's fg +
    /// content. `spaces=true` pads with " … ".
    #[inline(always)]
    fn write_segment<D: Display>(&mut self, seg: D, style: Style, spaces: bool) {
        let prev_sep_fg = match &self.last_style {
            Some(prev) => {
                // Order matters: set new bg first, then draw the old separator on top of it.
                let _ = write!(self.buffer, "{}{}{}", style.bg, prev.sep_fg, prev.sep);
                Some(prev.sep_fg)
            },
            None => {
                let _ = write!(self.buffer, "{}", style.bg);
                None
            },
        };

        // Skip the fg escape if we just wrote a separator in exactly this colour.
        if prev_sep_fg != Some(style.fg) {
            let _ = write!(self.buffer, "{}", style.fg);
        }

        // `let _ = ...` discards the `Result` — writing into a `String` is infallible.
        let _ = if spaces { write!(self.buffer, " {} ", seg) } else { write!(self.buffer, "{}", seg) };

        self.last_style = Some(style)
    }

    /// Append a segment padded with spaces (the common case).
    pub fn add_segment<D: Display>(&mut self, seg: D, style: Style) {
        self.write_segment(seg, style, true)
    }

    /// Append a segment with no surrounding spaces.
    pub fn add_short_segment<D: Display>(&mut self, seg: D, style: Style) {
        self.write_segment(seg, style, false)
    }

    /// Run a module, letting it append zero or more segments.
    pub fn add_module<M: Module>(&mut self, mut module: M) {
        module.append_segments(self)
    }

    /// Mutable access to the last segment's style, so a module can rewrite its trailing separator after the
    /// fact (used by `Cwd` to upgrade the last thin divider into a solid one).
    pub fn last_style_mut(&mut self) -> Option<&mut Style> {
        self.last_style.as_mut()
    }
}

impl fmt::Display for Powerline {
    /// Flush the buffer, the trailing separator, and a final `Reset` so the shell prompt does not bleed into
    /// user input.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.last_style {
            Some(Style { sep_fg, sep, .. }) => write!(f, "{}{}{}{}{}", self.buffer, Reset, sep_fg, sep, Reset),
            None => Ok(()),
        }
    }
}
