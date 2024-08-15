//! The core Ark datastructure.

use crate::types::attrs::Attrs;
use crate::types::ipr::IPR;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

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
///
/// These three channels are implemented as immutable, reference-counted
/// vectors. This is great for memory hygiene! Almost every possible
/// transformation you'd ever want to do on an Ark will leave one or two
/// channels unchanged, and create a new vector for the stuff that _is_
/// changing.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ark<C>(
    pub(crate) Rc<Vec<IPR>>,
    pub(crate) Rc<Vec<Attrs>>,
    pub(crate) Rc<Vec<C>>,
);

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
        &self.0
    }

    /// Internal attrs list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F+D.
    ///  - `ark.attrs()[N]` corresponds to `ark.paths()[N]`.
    pub fn attrs(&self) -> &Vec<Attrs> {
        &self.1
    }

    /// Internal contents list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F, not F+D.
    ///  - `ark.contents()[N]` corresponds to `ark.paths()[N]`.
    pub fn contents(&self) -> &Vec<C> {
        &self.2
    }

    /// Number of entries in this Ark.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Slap together a new Ark from the constituent pieces.
    ///
    /// Panics if length invariants aren't fulfilled.
    pub fn compose(paths: Rc<Vec<IPR>>, attrs: Rc<Vec<Attrs>>, contents: Rc<Vec<C>>) -> Self {
        assert!(paths.len() == attrs.len());
        assert!(paths.len() >= contents.len());
        Self(paths, attrs, contents)
    }

    /// Break an Ark into its constituent components, moving them.
    ///
    /// This is designed to pair with `compose` to allow you to reuse backing
    /// memory while doing transformations. Usually you'll only care about
    /// transforming one, maybe two of the three channels.
    pub fn decompose(self) -> (Rc<Vec<IPR>>, Rc<Vec<Attrs>>, Rc<Vec<C>>) {
        (self.0, self.1, self.2)
    }

    /// Create an empty Ark.
    ///
    /// Not as widely useful as you'd think, since Ark is efficient for bulk
    /// operations, and not a great type for incremental mutability. Usually
    /// you'll want to work with a list of `(path, attrs, Contents<C>)` tuples for
    /// poking around in little bits and pieces. These convert back and forth
    /// with Arks very easily.
    pub fn empty() -> Self {
        Self::compose(Rc::new(vec![]), Rc::new(vec![]), Rc::new(vec![]))
    }
}
