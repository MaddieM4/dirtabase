use crate::digest::Digest;
use crate::resource::Resource;

// A content-addressed store which can load or save Resources.
pub trait ResourceStore {
    type Err;

    fn load(&mut self, d: &Digest) -> Result<Option<Resource>, Self::Err>;
    fn save(&mut self, res: &Resource) -> Result<(), Self::Err>;

    // In some implementations, is faster than `load`.
    // Generic implementation relies on `load` however.
    fn exists(&mut self, d: &Digest) -> Result<bool, Self::Err> {
        Ok(self.load(d)?.is_some())
    }
}

pub trait LabelStore {
    type Err;

    fn load(&mut self, label: impl AsRef<[u8]>) -> Result<Option<Digest>, Self::Err>;
    fn save(&mut self, label: impl AsRef<[u8]>, d: &Digest) -> Result<(), Self::Err>;

    // In some implementations, is faster than `load`.
    // Generic implementation relies on `load` however.
    fn exists(&mut self, label: impl AsRef<[u8]>) -> Result<bool, Self::Err> {
        Ok(self.load(label)?.is_some())
    }
}

#[derive(PartialEq,Debug)]
pub enum StorageError<R,L> where R: ResourceStore, L: LabelStore {
    ResourceError(R::Err),
    LabelError(L::Err),
}

type StorageResult<T,R,L> = Result<T, StorageError<R,L>>;

pub struct Storage<R,L> where R: ResourceStore, L: LabelStore {
    resources: R,
    labels: L,
}

impl<R,L> Storage<R,L> where R: ResourceStore, L: LabelStore {
    pub fn new(resources: R, labels: L) -> Self {
        Storage { resources: resources, labels: labels }
    }

    pub fn load(&mut self, label: impl AsRef<[u8]>) -> StorageResult<Option<Vec<u8>>, R,L> {
        let d = match self.labels.load(label) {
            Ok(Some(d)) => d,
            Ok(None) => return Ok(None),
            Err(e) => return Err(StorageError::LabelError(e)),
        };
        match self.resources.load(&d) {
            Ok(x) => Ok(x.map(|rsc| rsc.body)),
            Err(e) => Err(StorageError::ResourceError(e))
        }
    }
    pub fn save(&mut self, label: impl AsRef<[u8]>, body: impl Into<Resource>) -> StorageResult<(), R,L> {
        let rsc: Resource = body.into();
        if let Err(e) = self.resources.save(&rsc) {
            return Err(StorageError::ResourceError(e))
        }
        if let Err(e) = self.labels.save(label, &rsc.digest) {
            return Err(StorageError::LabelError(e))
        }
        Ok(())
    }
}
