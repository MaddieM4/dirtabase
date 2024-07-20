//! Common stuff used by all storage backends.
//!
//! Some of these things are just simple re-exports of stdlib things, but any
//! custom types that get reused across multiple backends are also defined
//! and exported from here.

pub use crate::digest::Digest;
pub use std::io;
pub use std::io::ErrorKind::NotFound;
pub use std::path::{Path, PathBuf};

/// Ways that a RefName conversion might be invalid.
#[derive(Debug, PartialEq)]
pub enum RefNameError {
    /// Every ref name must begin with '@' as the first character.
    MustStartWithAmp,

    /// After the '@', all characters must be alphanumeric.
    InvalidCharacters,
}

/// Text in the form of "@abc123".
///
/// Storage backends mostly deal in immutable, hash-named objects: the CAS.
/// However, it's useful to have mutable, human-readable nicknames, which
/// point at a CAS object, but can be updated to point at a different CAS
/// object at any time.
///
/// For simplicity of implementation, reference names have a few strict
/// limitations. Hence, the logic to validate them!
#[derive(Debug, PartialEq)]
pub struct RefName(String);

impl RefName {
    pub fn new(name: impl AsRef<str>) -> Result<Self, RefNameError> {
        let name = name.as_ref();
        match name.chars().nth(0) {
            Some('@') => match name.chars().skip(1).all(|c| c.is_alphanumeric()) {
                true => Ok(Self(name.into())),
                false => Err(RefNameError::InvalidCharacters),
            },
            _ => Err(RefNameError::MustStartWithAmp),
        }
    }
}

impl AsRef<Path> for RefName {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn refname_new() {
        assert_eq!(RefName::new("foo"), Err(RefNameError::MustStartWithAmp));
        assert_eq!(RefName::new("@f/b"), Err(RefNameError::InvalidCharacters));
        assert_eq!(RefName::new("@foo"), Ok(RefName("@foo".into())));
    }

    #[test]
    fn refname_asref_path() {
        let rn = RefName::new("@foo").unwrap();
        let p = Path::new("/some/dir").join(rn.as_ref());
        assert_eq!(p.to_string_lossy(), "/some/dir/@foo");
    }
}
