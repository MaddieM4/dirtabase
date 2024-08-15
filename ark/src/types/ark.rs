//! The core types and nothing else.

use crate::types::attrs::Attrs;
use crate::types::ipr::IPR;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ark<C> {
    pub(crate) paths: Vec<IPR>,
    pub(crate) attrs: Vec<Attrs>,
    pub(crate) contents: Vec<C>,
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

    /// Easy conversion by content type. Not quite automatic though.
    fn translate<SRC>(src: Ark<SRC>) -> Self
    where
        C: From<SRC>,
    {
        let (paths, attrs, contents) = src.decompose();
        let contents: Vec<C> = contents.into_iter().map(|t| t.into()).collect();
        Self::compose(paths, attrs, contents)
    }
}

impl From<Ark<&str>> for Ark<Vec<u8>> {
    fn from(src: Ark<&str>) -> Self {
        Self::translate(src)
    }
}
impl From<Ark<&str>> for Ark<String> {
    fn from(src: Ark<&str>) -> Self {
        Self::translate(src)
    }
}
impl From<Ark<String>> for Ark<Vec<u8>> {
    fn from(src: Ark<String>) -> Self {
        Self::translate(src)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn conversions() {
        let str_ark: Ark<&str> = vec![
            ("/a", at! { N => "1" }, Contents::File("1")),
            ("/b", at! { N => "2" }, Contents::Dir),
            ("/c", at! { N => "3" }, Contents::File("3")),
            ("/d", at! { N => "4" }, Contents::Dir),
            ("/e", at! { N => "5" }, Contents::File("5")),
            ("/f", at! { N => "6" }, Contents::Dir),
        ]
        .into();

        let string_ark: Ark<String> = str_ark.clone().into();
        let ba1: Ark<Vec<u8>> = str_ark.clone().into();
        let ba2: Ark<Vec<u8>> = string_ark.into();
        assert_eq!(ba1, ba2);
    }
}
