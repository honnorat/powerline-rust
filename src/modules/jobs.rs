use std::env;
use std::marker::PhantomData;

use super::Module;
use crate::{Color, Powerline, Style};

pub struct Jobs<S: JobsScheme> {
    scheme: PhantomData<S>,
}

pub trait JobsScheme {
    const JOBS_BG: Color;
    const JOBS_FG: Color;
}

impl<S: JobsScheme> Jobs<S> {
    pub fn new() -> Jobs<S> {
        Jobs { scheme: PhantomData }
    }
}

impl<S: JobsScheme> Module for Jobs<S> {
    fn append_segments(&mut self, powerline: &mut Powerline) {
        if let Ok(job_count) = env::var("NUM_JOBS") {
            if job_count != "0" {
                powerline.add_segment(job_count, Style::simple(S::JOBS_FG, S::JOBS_BG))
            }
        }
}}
