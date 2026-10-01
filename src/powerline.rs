use std::fmt::{self, Display, Write};

use crate::modules::Module;
use crate::terminal::*;

/// Solid powerline separator glyph.
pub const SEP_SOLID: char = ''; // '\u{E0B0}'

/// Thin powerline separator glyph.
pub const SEP_THIN: char = ''; // '\u{E0B1}'

/// Foreground/background colours plus the separator glyph emitted *after* this segment.
#[derive(Clone, Copy)]
pub struct Style {
    pub fg: FgColor,
    pub bg: BgColor,
    pub sep: char,
    pub sep_fg: FgColor,
    pub bold: bool,
}

impl Style {
    /// Solid powerline separator (U+E0B0), separator colour = own background.
    pub fn simple(fg: Color, bg: Color) -> Self {
        Self { fg: fg.into(), bg: bg.into(), sep: SEP_SOLID, sep_fg: bg.into(), bold: false }
    }

    /// No separator glyph — a space sits between this segment and the next.
    pub fn nosep(fg: Color, bg: Color) -> Self {
        Self { fg: fg.into(), bg: bg.into(), sep: ' ', sep_fg: bg.into(), bold: false }
    }

    /// Custom separator glyph and colour (used e.g. for the thin CWD divider).
    pub fn special(fg: Color, bg: Color, sep: char, sep_fg: Color) -> Self {
        Self { fg: fg.into(), bg: bg.into(), sep, sep_fg: sep_fg.into(), bold: false }
    }

    /// Render this segment's content in bold. The bold attribute is scoped to the content only (turned off
    /// again right after), so it never bleeds into the separator or the following segment.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

/// Accumulating prompt buffer. Segments are appended left-to-right; the separator between segments is emitted
/// lazily when the *next* segment arrives (we need its background colour first).
pub struct Powerline {
    buffer: String,
    last_style: Option<Style>,
}

impl Powerline {
    pub fn new() -> Self {
        Self { buffer: String::with_capacity(512), last_style: None }
    }

    /// Emit the previous segment's separator (now that we know the new bg), then the new segment's fg +
    /// content. `spaces=true` pads with " … ".
    fn write_segment<D: Display>(&mut self, seg: D, style: Style, spaces: bool) {
        // Order matters: set new bg first, then draw the old separator on top of it.
        let _ = write!(self.buffer, "{}", style.bg);
        if let Some(prev) = self.last_style {
            let _ = write!(self.buffer, "{}{}", prev.sep_fg, prev.sep);
        }

        // Skip the fg escape if we just wrote a separator in exactly this colour.
        if self.last_style.is_none_or(|prev| prev.sep_fg != style.fg) {
            let _ = write!(self.buffer, "{}", style.fg);
        }

        // Bold is toggled on then back off around the content itself, so it never leaks into the
        // separator or a subsequent non-bold segment.
        if style.bold {
            let _ = write!(self.buffer, "{}", Bold(true));
        }

        // `let _ = ...` discards the `Result` — writing into a `String` is infallible.
        let _ = if spaces { write!(self.buffer, " {seg} ") } else { write!(self.buffer, "{seg}") };

        if style.bold {
            let _ = write!(self.buffer, "{}", Bold(false));
        }

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
}

impl Default for Powerline {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Powerline {
    /// Flush the buffer, the trailing separator, and a final `Reset` so the shell prompt does not
    /// bleed into user input.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.last_style {
            Some(Style { sep_fg, sep, .. }) => write!(f, "{}{}{}{}{}", self.buffer, Reset, sep_fg, sep, Reset),
            None => Ok(()),
        }
    }
}
