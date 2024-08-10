//! Common stuff used by all storage backends.
//!
//! Some of these things are just simple re-exports of stdlib things, but any
//! custom types that get reused across multiple backends are also defined
//! and exported from here.

pub use crate::digest::Digest;
pub use crate::label::Label;
pub use std::io;
pub use std::io::ErrorKind::NotFound;
pub use std::path::{Path, PathBuf};
