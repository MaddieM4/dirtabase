use crate::digest::Digest;
use crate::rootdata::RootData;

#[derive(Debug, PartialEq)]
pub enum StorageErr {
    NotFound,
    Engine(String),
}
pub type StorageRes<T> = Result<T, StorageErr>;
pub type Buffer = Vec<u8>;

pub trait Store {
    fn load(&mut self, d: &Digest) -> StorageRes<&Buffer>;
    fn save(&mut self, d: &Digest, b: &Buffer) -> StorageRes<()>;

    fn read_root(&mut self) -> StorageRes<&RootData>;
    fn replace_root(&mut self, previous: &RootData, next: &RootData) -> StorageRes<bool>;

    // In some implementations, is faster than `load`.
    // Generic implementation relies on `load` however.
    fn exists(&mut self, d: &Digest) -> StorageRes<bool> {
        match self.load(d) {
            Ok(_) => Ok(true),
            Err(StorageErr::NotFound) => Ok(false),
            Err(x) => Err(x),
        }
    }
}
