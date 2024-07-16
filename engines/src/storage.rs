use types::*;

pub trait Store {
    type Error;
    // type Result<T> = Result<T, Self::Error>;

    // Low-level mandatory API
    fn load(&mut self, d: &Digest) -> Result<Option<Buffer>, Self::Error>;
    fn save(&mut self, d: &Digest, b: &Buffer) -> Result<(), Self::Error>;
    fn read_root(&mut self) -> Result<RootData, Self::Error>;
    fn replace_root(&mut self, previous: RootData, next: RootData) -> Result<bool, Self::Error>;

    // Higher-level ergonomic API
    fn exists(&mut self, d: &Digest) -> Result<bool,Self::Error> {
        match self.load(d) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(x) => Err(x),
        }
    }

    fn store(&mut self, rsc: impl Into<Resource>) -> Result<Resource, Self::Error> {
        let rsc: Resource = rsc.into();
        self.save(&rsc.digest, &rsc.body)?;
        Ok(rsc)
    }
}
