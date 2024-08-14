//! The core types and nothing else.

use crate::attr::Attrs;

/// Internal Path Representation.
///
/// This will be something pickier later with its own invariants.
pub type IPR = String;

/// An enum we use to differentiate dirs vs files.
///
/// File content is represented flexibly, and can be anything consistent,
/// from in-memory strings to Triads that represent stored data. That's
/// the secret sauce for performance and clarity when it comes to tasks
/// like importing and exporting files from a store with massive parallelism.
#[derive(Debug, PartialEq)]
pub enum Contents<C> {
    Dir,
    File(C),
}

impl<C> Contents<C> {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Dir => true,
            Self::File(_) => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Dir => false,
            Self::File(_) => true,
        }
    }
}

/// Representation of an Archive.
///
/// Because this is generic, it can represent things like a directory on disk,
/// allowing us to convert an Ark of on-disk files into an Ark of imported files
/// in a simple, high-performance way. It obviates the need for things like a
/// stream API, and allows for a lot of tests to be done in-memory without disk.
///
/// The underlying format is an SOA approach, which you can inspect with:
///
///   - ark.paths()
///   - ark.attrs()
///   - ark.contents()
#[derive(Debug, PartialEq)]
pub struct Ark<C> {
    pub(super) paths: Vec<IPR>,
    pub(super) attrs: Vec<Attrs>,
    pub(super) contents: Vec<C>,
}

impl<C> Ark<C> {
    /// Internal paths list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F+D.
    ///  - There are no duplicate paths.
    ///  - All files come before all directories.
    ///  - Within each of those sections, paths are sorted.
    pub fn paths(&self) -> &Vec<IPR> {
        &self.paths
    }

    /// Internal attrs list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F+D.
    ///  - `ark.attrs()[N]` corresponds to `ark.paths()[N]`.
    pub fn attrs(&self) -> &Vec<Attrs> {
        &self.attrs
    }

    /// Internal contents list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F, not F+D.
    ///  - `ark.contents()[N]` corresponds to `ark.paths()[N]`.
    pub fn contents(&self) -> &Vec<C> {
        &self.contents
    }

    /// Slap together a new Ark from the constituent pieces.
    ///
    /// Panics if length invariants aren't fulfilled.
    pub fn compose(paths: Vec<IPR>, attrs: Vec<Attrs>, contents: Vec<C>) -> Self {
        assert!(paths.len() == attrs.len());
        assert!(paths.len() >= contents.len());
        Self {
            paths: paths,
            attrs: attrs,
            contents: contents,
        }
    }

    /// Break an Ark into its constituent components, moving them.
    ///
    /// This is designed to pair with `compose` to allow you to reuse backing
    /// memory while doing transformations. Usually you'll only care about
    /// transforming one, maybe two of the three channels.
    pub fn decompose(self) -> (Vec<IPR>, Vec<Attrs>, Vec<C>) {
        (self.paths, self.attrs, self.contents)
    }
}
