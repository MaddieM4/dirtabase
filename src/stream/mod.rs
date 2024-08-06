//! Lets you easily build or copy directory trees using a builder API.
//!
//! This is more naive than storage-to-storage transfers, and misses out on
//! some of the possible optimizations that those transfers can use, but these
//! are useful in their simple ruggedness. It's the only way to import and
//! export from domains where a CAS storage model isn't present, like local
//! directories on your filesystem.

pub mod core;
pub mod debug;
pub mod osdir;
pub mod archive;
