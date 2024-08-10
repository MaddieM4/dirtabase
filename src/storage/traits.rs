//! Traits that define the Storage API.
//!
//! Every module should define a `storage(...)` method (with appropriate params)
//! which produces a Storage object, as the entrypoint of the external API.
//!
//! ```
//! use tempfile::tempdir;
//! use dirtabase::digest::Digest;
//! use dirtabase::storage::traits::*;
//! use dirtabase::storage::simple;
//!
//! let dir = tempdir()?;
//! let store = simple::storage(&dir)?;
//! let digest = store.cas().write_buf("foo")?;
//!
//! assert_eq!(digest.to_hex(), Digest::from("foo").to_hex());
//! Ok::<(), std::io::Error>(())
//! ```

use crate::digest::Digest;
use crate::label::Label;
use std::io::{Read,Result,Cursor};

/// Content-addressed storage interface.
pub trait CAS {
    type Reader: Read;

    /// Get the contents of a resource within the store.
    fn read(&self, digest: &Digest) -> Result<Option<Self::Reader>>;

    /// Save a potentially new resource into the store.
    fn write(&self, reader: impl Read) -> Result<Digest>;

    /// Convenience method to write a buffer into the store.
    fn write_buf(&self, buf: impl AsRef<[u8]>) -> Result<Digest> {
        self.write(Cursor::new(buf))
    }
}

/// The part of a store that houses mutable labels.
///
/// It's worth noting that the interpretation of bytes within a label is, at
/// the raw storage level, entirely arbitrary and left as an exercise to the
/// caller. However, other parts of the Dirtabase codebase will use a specific
/// consistent format. The only assumption at the _storage_ level is that these
/// will be, in _some_ form, a **reasonably small** reference to something in
/// the CAS section of the same Storage.
pub trait Labels {
    /// Get the current value of a label.
    fn read(&self, name: &Label) -> Result<Vec<u8>>;

    /// Set the current value of a label.
    fn write(&self, name: &Label, value: impl AsRef<[u8]>) -> Result<()>;
}
