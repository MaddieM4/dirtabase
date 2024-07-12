use crate::traits::*;
use crate::digest::Digest;
use crate::resource::Resource;

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
    pub fn save(&mut self, label: impl AsRef<[u8]>, body: impl Into<Resource>) -> StorageRes<()> {
        let label: Label = label.as_ref().into();
        let rsc: Resource = body.into();
        self.save_resource(&rsc.digest, &rsc.body)?;
        self.save_label(&label, &rsc.digest)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memstore::*;

    #[test]
    fn storage() {
        let mut s = Storage::new(MemResources::new(), MemLabels::new());
        assert_eq!(s.load("some label"), Err(StorageErr::NotFound));

        s.save("some label", "some bytes").expect("In-memory save");
        assert_eq!(
            s.load("some label").unwrap(),
            &Vec::<u8>::from("some bytes")
        );
    }
}
