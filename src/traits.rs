use types::Digest;

#[derive(Debug, PartialEq)]
pub enum StorageErr {
    NotFound,
    Engine(String),
}
pub type StorageRes<T> = Result<T, StorageErr>;

pub trait Store<K, V> {
    fn load(&mut self, k: &K) -> StorageRes<&V>;
    fn save(&mut self, k: &K, v: &V) -> StorageRes<()>;

    // In some implementations, is faster than `load`.
    // Generic implementation relies on `load` however.
    fn exists(&mut self, k: &K) -> StorageRes<bool> {
        match self.load(k) {
            Ok(_) => Ok(true),
            Err(StorageErr::NotFound) => Ok(false),
            Err(x) => Err(x),
        }
    }
}

pub type Label = Vec<u8>;
pub type Buffer = Vec<u8>;

pub trait LabelStore: Store<Label, Digest> {}
pub trait ResourceStore: Store<Digest, Buffer> {}
