use sha2::{Digest as UpstreamDigest, Sha256};
const DIGEST_LENGTH: usize = 256 / 8;

#[derive(Debug,PartialEq,Eq,Hash)]
pub struct Digest {
    bytes: [u8; DIGEST_LENGTH],
}

impl Digest {
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }
}

impl<T> From<T> for Digest where T: AsRef<[u8]> {
    fn from(item: T) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(item);
        let bytes: [u8; DIGEST_LENGTH] = hasher.finalize().as_slice().try_into().unwrap();
        Self { bytes: bytes }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn digest_from_str() {
        let d = Digest::from("Hello world!");
        assert_eq!(
            d.to_hex(),
            "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a".to_string()
        );
    }

    #[test]
    fn digest_from_string() {
        let s: String = "Some text".into();
        let d = Digest::from(s);
        assert_eq!(
            d.to_hex(),
            "4c2e9e6da31a64c70623619c449a040968cdbea85945bf384fa30ed2d5d24fa3".to_string()
        );
    }
}
