use crate::archive::core::*;
use crate::storage::traits::CAS;
use std::io::{Cursor,Result};

// How on earth do we want to interact with a Storage?

fn archive_encode(ar: &Archive, f: Format, c: Compression) -> Result<Vec<u8>> {
    // Current support limitations
    assert!(f == Format::JSON);
    assert!(c == Compression::Plain);

    serde_json::to_vec(ar).map_err(|e| std::io::Error::other(e))
}

fn archive_decode(bytes: Vec<u8>, f: Format, c: Compression) -> Result<Archive> {
    // Current support limitations
    assert!(f == Format::JSON);
    assert!(c == Compression::Plain);

    serde_json::from_slice(bytes.as_ref()).map_err(|e| std::io::Error::other(e))
}

pub fn write_archive(ar: &Archive, f: Format, c: Compression, cas: impl CAS) -> Result<Digest> {
    // Turn `ar` into `bytes: Vec<u8>`
    let bytes = archive_encode(ar, f, c)?;

    // Make a Cursor on bytes
    let curs = Cursor::new(bytes);

    // Use that in cas.write()
    cas.write(curs)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn round_trip_encoding() {
        let before: Archive = vec![
            Entry(
                "/hello/world.txt".into(),
                Triad(Format::File, Compression::Plain, "some contents".into()),
                Attrs::new().set("MIME", "text/plain"),
            ),
        ];

        let bytes = archive_encode(&before, Format::JSON, Compression::Plain)
            .expect("Should not fail to serialize");
        assert!(bytes.len() > 0);
        
        let after = archive_decode(bytes, Format::JSON, Compression::Plain).expect("Should not fail to deserialize");
        assert_eq!(after, before);
    }
}
