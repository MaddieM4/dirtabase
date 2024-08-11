//! Simple datastore model.
//!
//! This is stored as an OS directory with a structure like:
//!
//! ```text
//! .dirtabase_db
//!  - labels/
//!    - @foo (arbitrary bytes, conventionally reference a CAS digest+format+compression)
//!    - @bar
//!    - tmp.27sd91 (tempfile that will be atomically renamed)
//!  - cas/
//!    - 12345678990909... (digest-named files)
//!    - ffffffffffffff...
//!    - eeeeeeeeeeeeee...
//!    - tmp.82789 (tempfile that will be renamed according to its hash)
//! ```
//!
//! What's important is that the Storage layer is totally parsing-ignorant.
//! This allows the many implementations to be simpler to write and understand
//! without thinking about too many distant moving parts.
//!
//! ```
//! use dirtabase::digest::Digest;
//! use dirtabase::storage;
//!
//! let store = storage::new_from_tempdir()?;
//! let digest = store.cas().write_buf("foo")?;
//!
//! assert_eq!(digest.to_hex(), Digest::from("foo").to_hex());
//! # Ok::<(), std::io::Error>(())
//! ```

use crate::digest::Digest;
use crate::label::Label;
use sha2::Digest as _;
use std::io::ErrorKind::NotFound;
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Implementation of the simple storage backend.
pub struct SimpleStorage<P>(P, SimpleCAS, SimpleLabels)
where
    P: AsRef<Path>;
impl<P> SimpleStorage<P>
where
    P: AsRef<Path>,
{
    /// Create a simple storage backend.
    pub fn new(path: P) -> io::Result<Self> {
        let cas = SimpleCAS::new(path.as_ref().join("cas"))?;
        let labels = SimpleLabels::new(path.as_ref().join("labels"))?;
        Ok(Self(path, cas, labels))
    }

    pub fn cas(&self) -> &SimpleCAS {
        &self.1
    }
    pub fn labels(&self) -> &SimpleLabels {
        &self.2
    }
}

/// Content-addressed storage in the Simple DB format.
pub struct SimpleCAS(PathBuf);
impl SimpleCAS {
    fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path: PathBuf = path.as_ref().into();
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    /// Get the contents of a resource within the store.
    pub fn read(&self, digest: &Digest) -> io::Result<Option<std::fs::File>> {
        let path = self.0.join(digest.to_hex());
        match std::fs::File::open(path) {
            Ok(f) => Ok(Some(f)),
            Err(e) => match e.kind() {
                NotFound => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Save a potentially new resource into the store.
    pub fn write(&self, mut reader: impl io::Read) -> io::Result<Digest> {
        let mut writer = NamedTempFile::new_in(&self.0)?;
        let mut hash = sha2::Sha256::new();
        // Copy data while building digest
        let mut buf = [0; 4096];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let bytes = &buf[..n];
            hash.update(bytes);
            writer.write(bytes)?;
        }
        // Finish up
        let raw = hash.finalize();
        let d = Digest::from_bytes(raw.as_slice().try_into().unwrap());
        writer.persist(self.0.join(d.to_hex()))?;
        Ok(d)
    }

    /// Convenience method to write a buffer into the store.
    pub fn write_buf(&self, buf: impl AsRef<[u8]>) -> io::Result<Digest> {
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
pub struct SimpleLabels(PathBuf);
impl SimpleLabels {
    fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path: PathBuf = path.as_ref().into();
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    /// Get the current value of a label.
    pub fn read(&self, name: &Label) -> io::Result<Vec<u8>> {
        match std::fs::read(&self.0.join(name.as_path())) {
            Ok(bytes) => Ok(bytes),
            Err(e) => match e.kind() {
                NotFound => Ok(vec![]),
                _ => Err(e),
            },
        }
    }

    /// Set the current value of a label.
    pub fn write(&self, name: &Label, value: impl AsRef<[u8]>) -> io::Result<()> {
        let dest = &self.0.join(name.as_path());
        let mut file = NamedTempFile::new_in(&self.0)?;
        file.write_all(value.as_ref())?;
        match file.persist(dest) {
            Ok(_) => Ok(()),
            Err(pe) => Err(pe.error),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn cas_read() -> io::Result<()> {
        let dir = tempdir()?;
        let store = SimpleStorage::new(&dir)?;
        let d: Digest = "some text".into();
        let path = dir.path().join("cas").join(d.to_hex());

        // No cas file, treated as no IO error but option is None
        assert!(store.cas().read(&d)?.is_none());

        // Artificially inject file
        std::fs::write(path, b"blah blah blah")?;
        let mut buf: Vec<u8> = vec![];
        store
            .cas()
            .read(&d)?
            .expect("file exists now")
            .read_to_end(&mut buf)?;
        assert_eq!(buf, b"blah blah blah");

        Ok(())
    }

    #[test]
    fn cas_write() -> io::Result<()> {
        let dir = tempdir()?;
        let store = SimpleStorage::new(&dir)?;
        let contents = "some text";
        let d: Digest = contents.into();
        let path = dir.path().join("cas").join(d.to_hex());

        // No cas file yet
        assert!(!path.exists());

        // Store into the CAS
        let d2 = store.cas().write(std::io::Cursor::new(contents))?;

        // Exists with expected contents
        assert_eq!(String::from_utf8(std::fs::read(path)?).unwrap(), contents);
        assert_eq!(d.to_hex(), d2.to_hex());

        Ok(())
    }

    #[test]
    fn lab_read() -> io::Result<()> {
        let dir = tempdir()?;
        let store = SimpleStorage::new(&dir)?;
        let lab = Label::new("@foo").unwrap();
        let path = dir.path().join("labels/@foo");

        // No label file but not an error, represented as empty array
        assert_eq!(store.labels().read(&lab)?, vec![0; 0]);

        // Artificially inject some contents
        std::fs::write(path, b"Some bytes")?;
        assert_eq!(store.labels().read(&lab)?, b"Some bytes");

        Ok(())
    }

    #[test]
    fn lab_write() -> io::Result<()> {
        let dir = tempdir()?;
        let store = SimpleStorage::new(&dir)?;
        let lab = Label::new("@foo").unwrap();
        let path = dir.path().join("labels/@foo");

        // Prior to write, file doesn't exist
        assert!(!path.exists());

        // After writing, contains the expected contents
        store.labels().write(&lab, b"Some contents")?;
        assert_eq!(std::fs::read(path)?, b"Some contents");

        Ok(())
    }
}
