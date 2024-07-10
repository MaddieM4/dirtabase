use crate::digest::Digest;
use crate::resource::Resource;

#[derive(Debug,PartialEq)]
pub enum LoadError<T> {
    NotFound,
    EngineError(T),
}

// A content-addressed store which can load or save Resources.
pub trait ResourceStore<T> where T: AsRef<[u8]> {
    type Err;

    fn load(&mut self, d: &Digest) -> Result<Resource<T>, LoadError<Self::Err>>;
    fn save(&mut self, res: Resource<T>) -> Result<(), Self::Err>;

    // In some implementations, is faster than `load`.
    // Generic implementation relies on `load` however.
    fn exists(&mut self, d: &Digest) -> Result<bool, Self::Err> {
        match self.load(d) {
            Ok(_) => Ok(true),
            Err(LoadError::NotFound) => Ok(false),
            Err(LoadError::EngineError(e)) => Err(e),
        }
    }
}

pub trait LabelStore {
    type Err;

    fn load(&mut self, label: impl AsRef<[u8]>) -> Result<Digest, LoadError<Self::Err>>;
    fn save(&mut self, label: impl AsRef<[u8]>, d: &Digest) -> Result<(), Self::Err>;

    // In some implementations, is faster than `load`.
    // Generic implementation relies on `load` however.
    fn exists(&mut self, label: impl AsRef<[u8]>) -> Result<bool, Self::Err> {
        match self.load(label) {
            Ok(_) => Ok(true),
            Err(LoadError::NotFound) => Ok(false),
            Err(LoadError::EngineError(e)) => Err(e),
        }
    }
}
