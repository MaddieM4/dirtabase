use types::*;
use crate::storage::*;
use std::path::{Path,PathBuf};
use std::fs;
use std::io::Result as Res;
use std::io::ErrorKind::NotFound;
use std::io::{Read,Write,Seek};
use file_lock::{FileLock,FileOptions};

// ----------------------------------------------------------------------------
// Simple DB Engine
// ----------------------------------------------------------------------------
pub struct Simple {
    path: PathBuf,
}
impl Simple {
    pub fn new(path: impl AsRef<Path>) -> Res<Self> {
        let buf: PathBuf = path.as_ref().into();
        std::fs::create_dir_all(Path::new(&buf).join("cas"))?;
        Ok(Self { path: buf })
    }
}
impl Store for Simple {
    type Error = std::io::Error;

    fn load(&mut self, d: &Digest) -> Res<Option<Buffer>> {
        let rsc_path = self.path.join("cas").join(d.to_hex());
        match fs::read(rsc_path) {
            Ok(buf) => Ok(Some(buf)),
            Err(e) => match e.kind() {
                NotFound => Ok(None),
                _ => Err(e),
            }
        }
    }

    fn save(&mut self, d: &Digest, b: &Buffer) -> Res<()> {
        let rsc_path = self.path.join("cas").join(d.to_hex());
        fs::write(rsc_path, b)
    }

    fn read_root(&mut self) -> Res<RootData> {
        let buf = match read_root(self.path.join("root.json")) {
            Ok(b) => b,
            Err(e) => match e.kind() {
                NotFound => return Ok(None),
                _ => Err(e)?,
            }
        };

        if buf.len() == 0 {
            Ok(None)
        } else {
            let spec: Spec = serde_json::from_slice(&buf).expect("Failed to parse JSON");
            Ok(Some(spec))
        }
    }

    fn replace_root(&mut self, previous: RootData, next: RootData) -> Res<bool> {
        let path = self.path.join("root.json");
        let buf_prev = match previous {
            Some(spec) => serde_json::to_vec(&spec).expect("Failed to serialize previous"),
            None => vec![],
        };
        let buf_next = match next {
            Some(spec) => serde_json::to_vec(&spec).expect("Failed to serialize next"),
            None => vec![],
        };
        replace_root(path, &buf_prev, &buf_next)
    }
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------
fn read_root(path: impl AsRef<Path>) -> Res<Vec<u8>> {
    let blocking = true;
    let options = FileOptions::new().read(true);
    let mut fl = FileLock::lock(path, blocking, options)?;
    let mut buf: Vec<u8> = vec![];
    fl.file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn replace_root(path: impl AsRef<Path>, previous: &Vec<u8>, next: &Vec<u8>) -> Res<bool> {
    let blocking = true;
    let options = FileOptions::new().read(true).create(true).write(true);
    let mut fl = FileLock::lock(path, blocking, options)?;
    let mut buf: Vec<u8> = vec![];
    fl.file.read_to_end(&mut buf)?;

    if &buf == previous {
        fl.file.rewind()?;
        fl.file.write_all(next)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new(test_name: &'static str) -> Simple {
        // TODO: Tempdir crate usage
        let path = Path::new("/tmp").join("dirtabase-test").join(test_name);
        if path.exists() {
            std::fs::remove_dir_all(&path).expect("Could not remove directory");
        }
        Simple::new(path).expect("Could not create path")
    }

    #[test]
    fn load() {
        let mut store = new("load");
        let d: Digest = "foo".into();
        assert_eq!(store.load(&d).unwrap(), None);
    }

    #[test]
    fn save() {
        let mut store = new("save");
        let b: Buffer = "foo".into();
        let d: Digest = (&b).into();
        assert_eq!(store.save(&d, &b).unwrap(), ());
        assert_eq!(store.load(&d).unwrap(), Some(b));
    }

    #[test]
    fn read_root() {
        let mut store = new("read_root");
        assert_eq!(store.read_root().unwrap(), None);
    }

    #[test]
    fn replace_root() {
        let mut store = new("replace_root");
        let abc = Spec {
            format: Format::JSON,
            compression: Compression::Plain,
            digest: "abc".into(),
        };
        let xyz = Spec {
            format: Format::JSON,
            compression: Compression::Plain,
            digest: "xyz".into(),
        };

        // Attempt to store with wrong previous, fails
        assert_eq!(store.replace_root(Some(abc.clone()), Some(xyz.clone())).unwrap(), false);
        assert_eq!(store.read_root().unwrap(), None);

        // Store with correct previous, succeeds
        assert_eq!(store.replace_root(None, Some(abc.clone())).unwrap(), true);
        assert_eq!(store.read_root().unwrap(), Some(abc.clone()));

        // Store one final version
        assert_eq!(store.replace_root(Some(abc.clone()), Some(xyz.clone())).unwrap(), true);
        assert_eq!(store.read_root().unwrap(), Some(xyz.clone()));
    }
}
