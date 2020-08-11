use std::fmt;
use std::fmt::Write as FmtWrite;

use crate::{modules::Module, terminal::*, R};

#[derive(Clone)]
pub struct Segment {
	pub val: String,
	pub fg: FgColor,
	pub bg: BgColor,
	pub sep: Option<char>,
	pub sep_col: FgColor,
}

impl Segment {
	pub fn simple<S: Into<String>>(val: S, fg: Color, bg: Color) -> Segment {
		Segment {
			val: val.into(),
			fg: fg.into_fg(),
			bg: bg.into_bg(),
			sep: Some('\u{E0B0}'),
			sep_col: bg.into_fg(),
		}
	}

	pub fn special<S: Into<String>>(val: S, fg: Color, bg: Color, sep: Option<char>, sep_col: Color) -> Segment {
		Segment {
			val: val.into(),
			fg: fg.into_fg(),
			bg: bg.into_bg(),
			sep,
			sep_col: sep_col.into_fg(),
		}
	}

	pub fn nosep<S: Into<String>>(val: S, fg: Color, bg: Color) -> Segment {
		Segment {
			val: val.into(),
			fg: fg.into_fg(),
			bg: bg.into_bg(),
			sep: None,
			sep_col: fg.into_fg(),
		}
	}
}

pub struct Powerline {
	segments: Vec<Segment>,
}

impl Powerline {
	pub fn new() -> Powerline {
		Powerline { segments: Vec::new() }
	}

	pub fn add_module(&mut self, mut part: impl Module) -> R<()> {
		part.append_segments(&mut self.segments)
	}

	pub fn add_segments(&mut self, new_segments: Vec<Segment>) {
		self.segments.extend(new_segments);
	}
}

impl fmt::Display for Powerline {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut iter = self.segments.iter().peekable();
		while let Some(seg) = iter.next() {
			let mut sep = String::new();
    		write!(&mut sep, "{}", seg.sep_col)?;
			if ! seg.sep.is_none() {
				sep.push(seg.sep.unwrap());
			}
			if let Some(next) = iter.peek() {
				write!(f, "{}{}{}{}{}", seg.fg, seg.bg, seg.val, next.bg, sep)?;
			} else {
				write!(f, "{}{}{}{}{}", seg.fg, seg.bg, seg.val, Reset, sep)?;
			}
		}
		write!(f, "{} ", Reset)
	}
}
