//! Defines the Sink trait.

use crate::attr::*;
use std::io::{Read, Result};
use std::path::Path;

/// The trait that all `dirtabase::stream::*::sink()` return types must fulfill.
///
/// This is a builder API, which can be used directly/inline, or by a `source()`
/// function. Sending directories is technically only necessary to set attributes
/// on them, or ensure a directory exists even when empty, as both `send_dir` and
/// `send_file` imply the automatic creation of any necessary parent directories.
///
/// Different Sinks will have different context-appropriate behavior for `finalize()`,
/// but it's typical for Sinks to behave in some kind of atomic manner, such that
/// the `finalize()` function makes the effects real. See `dirtabase::stream::osdir` for
/// a very practical and concrete example.
pub trait Sink where Self: Sized {
    type Receipt;

    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self>;
    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, r: impl Read) -> Result<Self>;
    fn finalize(self) -> Result<Self::Receipt>;
}
