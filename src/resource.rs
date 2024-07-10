// A resource is a series of bytes that can be stored or retrieved
// from a content-addressed store. It always has a precomputed digest.

use crate::digest::Digest;

struct Resource<T> where T: AsRef<[u8]> {
    pub digest: Digest,
    pub body: T,
}

impl<T> From<T> for Resource<T> where T: AsRef<[u8]> {
    fn from(item: T) -> Self {
        Resource {
            digest: Digest::from(&item),
            body: item,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_static_str() {
        let s = "Hello world!";
        let r = Resource::from(s);
        assert_eq!(r.body, s);
        assert_eq!(r.digest.to_hex(), Digest::from(s).to_hex());
    }

    #[test]
    fn from_string() {
        let s = "Hello world!".to_string();
        let sc = s.clone();
        let r = Resource::from(s); // Moves s
        assert_eq!(&r.body, &sc);
        assert_eq!(r.digest.to_hex(), Digest::from(&sc).to_hex());
    }

}
