use crate::digest::Digest;
use crate::resource::Resource;
use crate::traits::*;
use std::collections::HashMap;

type MemResources = HashMap<Digest, Vec<u8>>;
impl ResourceStore for MemResources {
    type Err = ();
    fn load(&mut self, d: &Digest) -> Result<Resource, LoadError<Self::Err>> {
        match self.get(d) {
            Some(s) => Ok(Resource::from(s.clone())),
            None => Err(LoadError::NotFound),
        }
    }
    fn save(&mut self, res: Resource) -> Result<(), Self::Err> {
        self.insert(res.digest, res.body);
        Ok(()) // Cannot fail
    }
}

type MemLabels = HashMap<Vec<u8>, Digest>;
impl LabelStore for MemLabels {
    type Err = ();
    fn load(&mut self, label: impl AsRef<[u8]>) -> Result<Digest, LoadError<Self::Err>> {
        let k: Vec<u8> = label.as_ref().into();
        match self.get(&k) {
            Some(d) => Ok(d.clone()),
            None => Err(LoadError::NotFound),
        }
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
        let d: Digest = "foo".into();
        assert_eq!(mr.load(&d), Err(LoadError::NotFound));
        assert_eq!(mr.exists(&d).unwrap(), false);

        mr.save("foo".to_string().into()).expect("Saving to memory should never fail");
        assert_eq!(mr.exists(&d).unwrap(), true);
        assert_eq!(mr.load(&d), Ok(Resource::from("foo".to_string())));
    }

    #[test]
    fn label_round_trip() {
        let mut ml = MemLabels::new();
        let label = "Some label";
        let d: Digest = "foo".into();
        assert_eq!(ml.load(label), Err(LoadError::NotFound));
        assert_eq!(ml.exists(label).unwrap(), false);

        ml.save(label, &d).expect("Saving to memory should never fail");
        assert_eq!(ml.exists(label).unwrap(), true);
        assert_eq!(ml.load(label), Ok(d));
    }
}
