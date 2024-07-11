use crate::digest::Digest;
use crate::resource::Resource;
use crate::traits::*;
use std::collections::HashMap;

type MemResources = HashMap<Digest, Vec<u8>>;
impl ResourceStore for MemResources {
    type Err = ();
    fn load(&mut self, d: &Digest) -> Result<Option<Resource>, Self::Err> {
        Ok(self.get(d).map(|ptr| ptr.into()))
    }
    fn save(&mut self, res: &Resource) -> Result<(), Self::Err> {
        self.insert(res.digest.clone(), res.body.clone());
        Ok(()) // Cannot fail
    }
}

type MemLabels = HashMap<Vec<u8>, Digest>;
impl LabelStore for MemLabels {
    type Err = ();
    fn load(&mut self, label: impl AsRef<[u8]>) -> Result<Option<Digest>, Self::Err> {
        let k: Vec<u8> = label.as_ref().into();
        Ok(self.get(&k).map(|ptr| ptr.clone()))
    }
    fn save(&mut self, label: impl AsRef<[u8]>, d: &Digest) -> Result<(), Self::Err> {
        self.insert(label.as_ref().into(), d.clone());
        Ok(()) // Cannot fail
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resource_round_trip() {
        let mut mr = MemResources::new();
        let rsc: Resource = "foo".to_string().into();
        let d = rsc.digest.clone();
        assert_eq!(mr.load(&d).unwrap(), None);
        assert_eq!(mr.exists(&d).unwrap(), false);

        mr.save(&rsc).expect("Saving to memory should never fail");
        assert_eq!(mr.exists(&d).unwrap(), true);
        assert_eq!(mr.load(&d).unwrap(), Some(Resource::from("foo".to_string())));
    }

    #[test]
    fn label_round_trip() {
        let mut ml = MemLabels::new();
        let label = "Some label";
        let d: Digest = "foo".into();
        assert_eq!(ml.load(label).unwrap(), None);
        assert_eq!(ml.exists(label).unwrap(), false);

        ml.save(label, &d).expect("Saving to memory should never fail");
        assert_eq!(ml.exists(label).unwrap(), true);
        assert_eq!(ml.load(label).unwrap(), Some(d));
    }

    #[test]
    fn storage() {
        let mut s = Storage::new(MemResources::new(), MemLabels::new());
        assert_eq!(s.load("some label"), Ok(None));

        s.save("some label", "some bytes").expect("In-memory save");
        assert_eq!(s.load("some label").unwrap(), Some(Vec::<u8>::from("some bytes")));
    }
}
