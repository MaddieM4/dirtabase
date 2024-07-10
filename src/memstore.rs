use crate::digest::Digest;
use crate::resource::Resource;
use crate::traits::*;
use std::collections::HashMap;

type MemResources = HashMap<Digest, String>;
impl ResourceStore<String> for MemResources {
    type Err = ();
    fn load(&mut self, d: &Digest) -> Result<Resource<String>, LoadError<Self::Err>> {
        match self.get(d) {
            Some(s) => Ok(Resource::from(s.clone())),
            None => Err(LoadError::NotFound),
        }
    }
    fn save(&mut self, res: Resource<String>) -> Result<(), Self::Err> {
        self.insert(res.digest, res.body);
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
}
