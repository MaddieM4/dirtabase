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

#[derive(PartialEq, Debug)]
pub enum StorageError<R, L>
where
    R: ResourceStore,
    L: LabelStore,
{
    ResourceError(R::Err),
    LabelError(L::Err),
}
type StorageResult<T, R, L> = Result<T, StorageError<R, L>>;

pub struct Storage<R, L>
where
    R: ResourceStore,
    L: LabelStore,
{
    resources: R,
    labels: L,
}

impl<R, L> Storage<R, L>
where
    R: ResourceStore,
    L: LabelStore,
{
    pub fn new(resources: R, labels: L) -> Self {
        Storage {
            resources: resources,
            labels: labels,
        }
    }

    fn load_resource(&mut self, d: &Digest) -> StorageResult<Option<Resource>, R, L> {
        Ok(self
            .resources
            .load(d)
            .map_err(|e| StorageError::ResourceError(e))?)
    }
    fn save_resource(&mut self, rsc: &Resource) -> StorageResult<(), R, L> {
        Ok(self
            .resources
            .save(rsc)
            .map_err(|e| StorageError::ResourceError(e))?)
    }

    fn load_label(&mut self, label: impl AsRef<[u8]>) -> StorageResult<Option<Digest>, R, L> {
        Ok(self
            .labels
            .load(label)
            .map_err(|e| StorageError::LabelError(e))?)
    }
    fn save_label(&mut self, label: impl AsRef<[u8]>, d: &Digest) -> StorageResult<(), R, L> {
        Ok(self
            .labels
            .save(label, d)
            .map_err(|e| StorageError::LabelError(e))?)
    }

    pub fn load(&mut self, label: impl AsRef<[u8]>) -> StorageResult<Option<Vec<u8>>, R, L> {
        let d = self.load_label(label)?;
        if let None = d {
            return Ok(None);
        }
        self.load_resource(&d.unwrap())
            .map(|opt| opt.map(|rsc| rsc.body.clone()))
    }
    pub fn save(
        &mut self,
        label: impl AsRef<[u8]>,
        body: impl Into<Resource>,
    ) -> StorageResult<(), R, L> {
        let rsc: Resource = body.into();
        self.save_resource(&rsc)?;
        self.save_label(label, &rsc.digest)
    }
}
