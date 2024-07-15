use types::*;
use crate::storage::*;
use std::collections::HashMap;

pub struct Memory {
    root: RootData,
    cas: HashMap<Digest, Vec<u8>>,
}

impl Memory {
    pub fn new() -> Self {
        Self { root: None, cas: HashMap::new() }
    }
}

impl Store for Memory {
    fn load(&mut self, d: &Digest) -> StorageRes<&Buffer> {
        self.cas.get(d).ok_or(StorageErr::NotFound)
    }

    fn save(&mut self, d: &Digest, b: &Buffer) -> StorageRes<()> {
        self.cas.insert(d.clone(), b.clone());
        Ok(())
    }

    fn read_root(&mut self) -> StorageRes<RootData> {
        Ok(self.root.clone())
    }

    fn replace_root(&mut self, previous: RootData, next: RootData) -> StorageRes<bool> {
        if self.root == previous {
            self.root = next;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load() {
        let mut store = Memory::new();
        let d: Digest = "foo".into();
        assert_eq!(store.load(&d), Err(StorageErr::NotFound));
    }

    #[test]
    fn save() {
        let mut store = Memory::new();
        let b: Buffer = "foo".into();
        let d: Digest = (&b).into();

        assert_eq!(store.save(&d, &b), Ok(()) );
        assert_eq!(store.load(&d), Ok(&b));
    }

    #[test]
    fn read_root() {
        let mut store = Memory::new();
        assert_eq!(store.read_root(), Ok(None));
    }

    #[test]
    fn replace_root() {
        let mut store = Memory::new();
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
        assert_eq!(store.replace_root(Some(abc.clone()), Some(xyz.clone())), Ok(false));
        assert_eq!(store.read_root(), Ok(None));

        // Store with correct previous, succeeds
        assert_eq!(store.replace_root(None, Some(abc.clone())), Ok(true));
        assert_eq!(store.read_root(), Ok(Some(abc.clone())));

        // Store one final version
        assert_eq!(store.replace_root(Some(abc.clone()), Some(xyz.clone())), Ok(true));
        assert_eq!(store.read_root(), Ok(Some(xyz.clone())));
    }
}

