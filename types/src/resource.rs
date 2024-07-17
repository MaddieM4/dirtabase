use crate::digest::Digest;
use crate::primitives::Buffer;
use std::path::{Path,PathBuf};

pub trait Resource {
    type Error;

    // Required methods
    fn contents(&self) -> Result<Buffer, Self::Error>;

    // Provided methods
    fn digest(&self) -> Result<Digest, Self::Error> {
        let d: Digest = self.contents()?.into();
        Ok(d)
    }
}

#[derive(Debug,PartialEq)]
pub struct Transient(Digest, Buffer);
impl<T> From<T> for Transient where T: AsRef<[u8]> {
    fn from(item: T) -> Self {
        Self(Digest::from(&item), item.as_ref().into())
    }
}
impl Resource for Transient {
    type Error = ();

    fn digest(&self) -> Result<Digest, Self::Error> {
        Ok(self.0.clone())
    }
    fn contents(&self) -> Result<Buffer, Self::Error> {
        Ok(self.1.clone())
    }
}

pub struct File(PathBuf);
impl File {
    pub fn new(p: impl AsRef<Path>) -> Self {
        Self(p.as_ref().into())
    }
}
impl Resource for File {
    type Error = std::io::Error;
    fn contents(&self) -> Result<Buffer, Self::Error> {
        std::fs::read(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn transient_from_static_str() {
        let s = "Hello world!";
        let r = Transient::from(s);
        assert_eq!(r.contents().unwrap(), Vec::<u8>::from(s));
        assert_eq!(r.digest().unwrap().to_hex(), Digest::from(s).to_hex());
    }

    #[test]
    fn transient_from_string() {
        let s = "Hello world!".to_string();
        let sc: Buffer = s.clone().into();
        let r = Transient::from(s); // Moves s
        assert_eq!(r.contents().unwrap(), sc);
        assert_eq!(r.digest().unwrap().to_hex(), Digest::from(&sc).to_hex());
    }

    #[test]
    fn file() {
        let f = File::new("src/resource.rs");
        let d = f.digest().expect("Load this file");
        let c = f.contents().expect("Load this file");
        assert_eq!(Digest::from(c), d);
    }
}
