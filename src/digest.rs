//! Compute cryptographic hash digests of content, in a way where you'd
//! have to go very explicitly out of your way to fake a hash or get it
//! wrong.
//!
//! ```
//! use dirtabase::digest::Digest;
//!
//! let d: Digest = "foo".into();
//! println!("{}", d.to_hex());
//! ```

// Don't pollute namespace, just make sure we're loading some traits
use sha2::Digest as _;
use hex::ToHex;

/// Anything that implements Cruncher can be used by DigestAny.
pub trait Cruncher {
    type Output: AsRef<[u8]>;
    fn crunch(data: impl AsRef<[u8]>) -> Self::Output;
}

impl Cruncher for sha2::Sha256 {
    type Output = [u8; 256/8];
    fn crunch(data: impl AsRef<[u8]>) -> Self::Output {
        Self::digest(data).as_slice().try_into().unwrap()
    }
}
impl Cruncher for sha2::Sha512 {
    type Output = [u8; 512/8];
    fn crunch(data: impl AsRef<[u8]>) -> Self::Output {
        Self::digest(data).as_slice().try_into().unwrap()
    }
}

/// Generic digest implementation.
///
/// This is used to make the "real" types
/// you'd use every day, particularly Digest. This flexibility should be
/// somewhat helpful if Sha256 ever proves inadequate, which isn't likely
/// in the _near_ future, but is plausible on a long enough timescale.
pub struct DigestAny<C> where C: Cruncher {
    bytes: C::Output,
}
impl<C> DigestAny<C> where C: Cruncher {
    /// Human-friendly representation of digest bytes.
    pub fn to_hex(&self) -> String {
        self.encode_hex()
    }

    /// Machine-friendly borrow of digest bytes.
    pub fn to_bytes(&self) -> &C::Output {
        &self.bytes
    }

    /// Import from some external source. Be careful to preserve invariants!
    pub fn from_bytes(bytes: C::Output) -> Self {
        Self { bytes: bytes }
    }
}
impl<T,C> From<T> for DigestAny<C> where C: Cruncher, T: AsRef<[u8]> {
    fn from(data: T) -> Self {
        Self { bytes: C::crunch(data) }
    }
}
impl<C> ToHex for DigestAny<C> where C: Cruncher, C::Output: ToHex {
    fn encode_hex<T: FromIterator<char>>(&self) -> T {
        self.bytes.encode_hex()
    }
    fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
        self.bytes.encode_hex_upper()
    }
}
impl<C> std::fmt::Debug for DigestAny<C> where C: Cruncher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Digest({:?})", self.to_hex())
    }
}

pub type DigestSha256 = DigestAny<sha2::Sha256>;
pub type DigestSha512 = DigestAny<sha2::Sha512>;

/// The default Digest type used throughout Dirtabase.
pub type Digest = DigestSha256;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default() {
        let d = Digest::from("Hello world!");
        assert_eq!(
            &d.to_hex(),
            "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a"
        );
        assert_eq!(d.to_bytes()[0..3], [192, 83, 94]);
    }

    #[test]
    fn sha512() {
        let d = DigestSha512::from("Hello world!");
        assert_eq!(
            &d.to_hex(),
            "f6cde2a0f819314cdde55fc227d8d7dae3d28cc556222a0a8ad66d91ccad4aad6094f517a2182360c9aacf6a3dc323162cb6fd8cdffedb0fe038f55e85ffb5b6"
        );
        assert_eq!(d.to_bytes()[0..3], [246, 205, 226]);
    }
}
