use std::{env, marker::PhantomData};

use super::Module;
use crate::{powerline::Segment, terminal::Color, R};

pub struct ExitCode<S: ExitCodeScheme> {
	scheme: PhantomData<S>,
}

pub trait ExitCodeScheme {
	const EXIT_CODE_BG: Color;
	const EXIT_CODE_FG: Color;
}

impl<S: ExitCodeScheme> ExitCode<S> {
	pub fn new() -> ExitCode<S> {
		ExitCode { scheme: PhantomData }
	}
}

impl<S: ExitCodeScheme> Module for ExitCode<S> {
	fn append_segments(&mut self, segments: &mut Vec<Segment>) -> R<()> {
		let args: Vec<String> = env::args().collect();
		let exit_code = args[1].trim().parse::<u32>().unwrap();

		if exit_code != 0 {
			let (fg, bg) = (S::EXIT_CODE_FG, S::EXIT_CODE_BG);
			segments.push(Segment::simple(format!(" {} ", exit_code), fg, bg));
		}

		Ok(())
	}
}
