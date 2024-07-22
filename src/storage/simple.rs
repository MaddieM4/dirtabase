//! Simple datastore model.
//!
//! This is stored as an OS directory with a structure like:
//!
//! ```text
//! .dirtabase_db
//!  - lab/
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

use crate::storage::core::*;
use tempfile::NamedTempFile;
use std::io::Write;
use sha2::Digest as _;

/// Content-addressed storage in the Simple DB format.
pub struct SimpleCAS(PathBuf);
impl SimpleCAS {
    fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path: PathBuf = path.as_ref().into();
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }
}
impl CAS for SimpleCAS {
    type Reader = std::fs::File;

    fn read(&self, digest: &Digest) -> io::Result<Option<Self::Reader>> {
        let path = self.0.join(digest.to_hex()); match std::fs::File::open(path) {
            Ok(f) => Ok(Some(f)),
            Err(e) => match e.kind() {
                NotFound => Ok(None),
                _ => Err(e)
            }
        }
    }
    fn write(&self, mut reader: impl io::Read) -> io::Result<Digest> {
        let mut writer = NamedTempFile::new_in(&self.0)?;
        let mut hash = sha2::Sha256::new();
        // Copy data while building digest
        let mut buf = [0; 4096];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break
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
}

pub fn storage(path: impl AsRef<Path>) -> io::Result<Simple> {
    Simple::new(path)
}

pub struct Simple {
    root: PathBuf,
    cas: SimpleCAS,
}

impl Simple {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let buf: PathBuf = path.as_ref().into();
        std::fs::create_dir_all(Path::new(&buf).join("lab"))?;
        let cas = SimpleCAS::new(&buf.join("cas"))?;
        Ok(Self { root: buf, cas: cas })
    }

    pub fn cas(&self) -> &SimpleCAS { &self.cas }

    fn lab_path(&self, name: impl AsRef<str>) -> PathBuf {
        let label = Label::new(name.as_ref()).expect("Invalid label name");
        self.root.join("lab").join(label.as_path())
    }

    pub fn lab_read(&self, name: impl AsRef<str>) -> io::Result<Vec<u8>> {
        match std::fs::read(self.lab_path(name)) {
            Ok(bytes) => Ok(bytes),
            Err(e) => match e.kind() {
                NotFound => Ok(vec![]),
                _ => Err(e),
            }
        }
    }

    pub fn lab_write(&self, name: impl AsRef<str>, value: impl AsRef<[u8]>) -> io::Result<()> { let dest = self.lab_path(name);
        let mut file = NamedTempFile::new_in(self.root.join("lab"))?;
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
    use tempdir::TempDir;
    use std::io::Read;

    #[test]
    fn lab_read() -> io::Result<()> {
        let dir = TempDir::new("dirtabase")?;
        let store = Simple::new(&dir)?;
        let path = dir.path().join("lab/@foo");

        // No label file but not an error, represented as empty array
        assert_eq!(store.lab_read("@foo")?, vec![0;0]);

        // Artificially inject some contents
        std::fs::write(path, b"Some bytes")?;
        assert_eq!(store.lab_read("@foo")?, b"Some bytes");

        Ok(())
    }

    #[test]
    fn lab_write() -> io::Result<()> {
        let dir = TempDir::new("dirtabase")?;
        let store = Simple::new(&dir)?;
        let path = dir.path().join("lab/@foo");

        // Prior to write, file doesn't exist
        assert!(!path.exists());

        // After writing, contains the expected contents
        store.lab_write("@foo", b"Some contents")?;
        assert_eq!(std::fs::read(path)?, b"Some contents");

        Ok(())
    }

    #[test]
    fn cas_read() -> io::Result<()> {
        let dir = TempDir::new("dirtabase")?;
        let store = Simple::new(&dir)?;
        let d: Digest = "some text".into();
        let path = dir.path().join("cas").join(d.to_hex());

        // No cas file, treated as no IO error but option is None
        assert!(store.cas().read(&d)?.is_none());

        // Artificially inject file
        std::fs::write(path, b"blah blah blah")?;
        let mut buf: Vec<u8> = vec![];
        store.cas().read(&d)?.expect("file exists now").read_to_end(&mut buf)?;
        assert_eq!(buf, b"blah blah blah");

        Ok(())
    }

    #[test]
    fn cas_write() -> io::Result<()> {
        let dir = TempDir::new("dirtabase")?;
        let store = Simple::new(&dir)?;
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
}
