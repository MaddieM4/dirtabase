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
