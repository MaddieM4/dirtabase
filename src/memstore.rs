use types::Digest;
use crate::traits::*;
use std::collections::HashMap;

// Cannot have engine failures, but fits the engine-fallible trait
impl<K,V> Store<K, V> for HashMap<K,V> where K: Eq + std::hash::Hash + Clone, V: Clone {
    fn load(&mut self, k: &K) -> StorageRes<&V> {
        self.get(k).ok_or(StorageErr::NotFound)
    }
    fn save(&mut self, k: &K, v: &V) -> StorageRes<()> {
        self.insert(k.clone(), v.clone());
        Ok(()) // Cannot fail
    }
}

pub type MemLabels = HashMap<Vec<u8>, Digest>;
impl LabelStore for MemLabels {}

pub type MemResources = HashMap<Digest, Vec<u8>>;
impl ResourceStore for MemResources {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resource_round_trip() {
        let mut mr = MemResources::new();
        let bytes: Vec<u8> = vec![1,2,3,4];
        let d: Digest = (&bytes).into();

        assert_eq!(mr.load(&d), Err(StorageErr::NotFound));
        assert_eq!(mr.exists(&d).unwrap(), false);

        mr.save(&d, &bytes).expect("Saving to memory should never fail");
        assert_eq!(mr.exists(&d).unwrap(), true);
        assert_eq!(mr.load(&d).unwrap(), &bytes);
    }

    #[test]
    fn label_round_trip() {
        let mut ml = MemLabels::new();
        let label: Vec<u8> = "Some label".into();
        let d: Digest = "foo".into();
        assert_eq!(ml.load(&label), Err(StorageErr::NotFound));
        assert_eq!(ml.exists(&label).unwrap(), false);

        ml.save(&label, &d).expect("Saving to memory should never fail");
        assert_eq!(ml.exists(&label).unwrap(), true);
        assert_eq!(ml.load(&label).unwrap(), &d);
    }
}
