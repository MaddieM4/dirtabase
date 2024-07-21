//! Simple datastore model
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

pub struct Simple {
    root: PathBuf,
}

impl Simple {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let buf: PathBuf = path.as_ref().into();
        std::fs::create_dir_all(Path::new(&buf).join("lab"))?;
        std::fs::create_dir_all(Path::new(&buf).join("cas"))?;
        Ok(Self { root: buf })
    }

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

    fn cas_path(&self, digest: &Digest) -> PathBuf {
        self.root.join("cas").join(digest.to_hex())
    }

    pub fn cas_read(&self, digest: &Digest) -> io::Result<Option<std::fs::File>> {
        let path = self.cas_path(digest);
        match std::fs::File::open(path) {
            Ok(f) => Ok(Some(f)),
            Err(e) => match e.kind() {
                NotFound => Ok(None),
                _ => Err(e)
            }
        }
    }
    pub fn cas_write(&self) -> io::Result<CasWriter> {
        CasWriter::new(&self.root)
    }
}

use sha2::Digest as _;
pub struct CasWriter {
    hash: sha2::Sha256, // TODO: Improve this :(
    file: NamedTempFile,
    root: PathBuf,
}
impl CasWriter {
    fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let dir = root.as_ref().join("cas");
        Ok(Self {
            hash: sha2::Sha256::new(),
            file: NamedTempFile::new_in(&dir)?,
            root: dir.into(),
        })
    }
    fn finish(self) -> io::Result<Digest> {
        let raw = self.hash.finalize();
        let d = Digest::from_bytes(raw.as_slice().try_into().unwrap());
        self.file.persist(self.root.join(d.to_hex()))?;
        Ok(d)
    }
}
impl Write for CasWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hash.update(buf);
        self.file.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
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
        assert!(store.cas_read(&d)?.is_none());

        // Artificially inject file
        std::fs::write(path, b"blah blah blah")?;
        let mut buf: Vec<u8> = vec![];
        store.cas_read(&d)?.expect("file exists now").read_to_end(&mut buf)?;
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
        let mut cw = store.cas_write()?;
        cw.write_all(contents.as_ref())?;
        assert!(!path.exists()); // EVEN NOW, DOES NOT EXIST AT FINAL LOCATION!
        let d2 = cw.finish()?;

        // Exists with expected contents
        assert_eq!(String::from_utf8(std::fs::read(path)?).unwrap(), contents);
        assert_eq!(d.to_hex(), d2.to_hex());

        Ok(())
    }
}
