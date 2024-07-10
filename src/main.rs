use sha2::{Digest as UpstreamDigest, Sha256};
const DIGEST_LENGTH: usize = 256 / 8;

// CAS = Content-addressed store
// -----------------------------
//
// Files are named according to the the hash of the contents. Files with equal
// contents will have the same name and naturally be deduplicated (only stored
// once). Even a slight difference in content will make the name different.
//
//
// Label Storage = Mutable
// -----------------------
//
// Labels map a human-readable name to some hash that can be found in the CAS.
// Labels can change over time, allowing a new version of a file or directory
// to become canon.

pub struct Digest {
    bytes: [u8; DIGEST_LENGTH],
}

impl Digest {
    fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }
}

impl From<&str> for Digest {
    fn from(item: &str) -> Self {
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
}

fn main() {
    println!("Hello, world!");
}
