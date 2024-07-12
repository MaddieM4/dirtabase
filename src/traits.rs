use crate::digest::Digest;
use crate::resource::Resource;

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

    fn load_resource(&mut self, d: &Digest) -> StorageRes<&Buffer> {
        self.resources.load(d)
    }
    fn save_resource(&mut self, d: &Digest, body: &Buffer) -> StorageRes<()> {
        self.resources.save(d, body)
    }

    fn load_label(&mut self, label: &Label) -> StorageRes<&Digest> {
        self.labels.load(label)
    }
    fn save_label(&mut self, label: &Label, d: &Digest) -> StorageRes<()> {
        self.labels.save(label, d)
    }

    pub fn load(&mut self, label: impl AsRef<[u8]>) -> StorageRes<&Buffer> {
        let label: Label = label.as_ref().into();
        let digest: Digest = self.load_label(&label)?.clone();
        self.load_resource(&digest)
    }
    pub fn save(
        &mut self,
        label: impl AsRef<[u8]>,
        body: impl Into<Resource>,
    ) -> StorageRes<()> {
        let label: Label = label.as_ref().into();
        let rsc: Resource = body.into();
        self.save_resource(&rsc.digest, &rsc.body)?;
        self.save_label(&label, &rsc.digest)
    }
}
