//! A mutable `@placeholder` in a storage backend.
//!
//! Storage backends mostly deal in immutable, hash-named objects: the CAS.
//! However, it's useful to have mutable, human-readable nicknames, which
//! point at a CAS object, but can be updated to point at a different CAS
//! object at any time.
//!
//! For simplicity of implementation, label names have a few strict
//! limitations. Hence, the logic to validate them!
//!
//! ```
//! use dirtabase::label::{Label,Error};
//!
//! assert_eq!(Label::new("invalid"), Err(Error::MustStartWithAmp));
//! assert_eq!(Label::new("@valid").unwrap().as_str(), "@valid");
//! ```

use std::path::Path;

/// Ways that a Label may be invalid.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Every ref name must begin with '@' as the first character.
    MustStartWithAmp,

    /// After the '@', all characters must be alphanumeric.
    InvalidCharacters,
}

/// Text in the form of "@abc123".
#[derive(Debug, PartialEq)]
pub struct Label(String);

impl Label {
    /// Create a Label from some sort of text, validating it.
    pub fn new(name: impl AsRef<str>) -> Result<Self, Error> {
        let name = name.as_ref();
        match name.chars().nth(0) {
            Some('@') => match name.chars().skip(1).all(|c| c.is_alphanumeric()) {
                true => Ok(Self(name.into())),
                false => Err(Error::InvalidCharacters),
            },
            _ => Err(Error::MustStartWithAmp),
        }
    }

    /// Yields an immutable Path reference.
    pub fn as_path(&self) -> &Path { self.0.as_ref() }

    /// Yields an immutable str reference.
    pub fn as_str(&self) -> &str { self.0.as_ref() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        assert_eq!(Label::new("foo"), Err(Error::MustStartWithAmp));
        assert_eq!(Label::new("@f/b"), Err(Error::InvalidCharacters));
        assert_eq!(Label::new("@foo"), Ok(Label("@foo".into())));
    }

    #[test]
    fn asref_path() {
        let rn = Label::new("@foo").unwrap();
        let p = Path::new("/some/dir").join(rn.as_path());
        assert_eq!(p.to_string_lossy(), "/some/dir/@foo");
    }

    #[test]
    fn asref_str() {
        let rn = Label::new("@foo").unwrap();
        assert_eq!(rn.as_str(), "@foo");
    }
}
