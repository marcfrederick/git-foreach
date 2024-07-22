#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub use error::{Error, Result};
pub use walk::{Walk, WalkBuilder};

mod error;
mod walk;
