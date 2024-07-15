use types::*;

#[derive(Debug, PartialEq)]
pub enum StorageErr {
    NotFound,
    Engine(String),
}
pub type StorageRes<T> = Result<T, StorageErr>;

pub trait Store {
    // Low-level mandatory API
    fn load(&mut self, d: &Digest) -> StorageRes<&Buffer>;
    fn save(&mut self, d: &Digest, b: &Buffer) -> StorageRes<()>;
    fn read_root(&mut self) -> StorageRes<RootData>;
    fn replace_root(&mut self, previous: RootData, next: RootData) -> StorageRes<bool>;

    // Higher-level ergonomic API
    fn exists(&mut self, d: &Digest) -> StorageRes<bool> {
        match self.load(d) {
            Ok(_) => Ok(true),
            Err(StorageErr::NotFound) => Ok(false),
            Err(x) => Err(x),
        }
    }
}
