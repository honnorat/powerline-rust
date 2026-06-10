//! Library backing the `powerline` binary. Customisation happens at compile time: you pick which `Module`s to
//! compose and which `…Scheme` (theme) to parametrise them over — there is no runtime config to keep startup
//! under ~10 ms.

pub mod modules;
pub mod powerline;
pub mod terminal;
pub mod theme;

// `pub(crate)` = visible inside this crate only, not part of the public API.
pub(crate) mod utils;

pub use crate::powerline::{Powerline, Style};
pub use crate::terminal::Color;
