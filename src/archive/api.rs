use crate::archive::core::*;
use crate::storage::traits::CAS;
use std::io::{Cursor,Result};

// How on earth do we want to interact with a Storage?

pub fn archive_encode(ar: &Archive, _f: ArchiveFormat, _c: Compression) -> Result<Vec<u8>> {
    serde_json::to_vec(ar).map_err(|e| std::io::Error::other(e))
}

pub fn archive_decode(bytes: Vec<u8>, _f: ArchiveFormat, _c: Compression) -> Result<Archive> {
    serde_json::from_slice(bytes.as_ref()).map_err(|e| std::io::Error::other(e))
}

pub fn write_archive(ar: &Archive, f: ArchiveFormat, c: Compression, cas: impl CAS) -> Result<Digest> {
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
            Entry::File {
                path: "/hello/world.txt".into(),
                compression: Compression::Plain,
                digest: "some contents".into(),
                attrs: Attrs::new().set("MIME", "text/plain"),
            },
        ];

        let bytes = archive_encode(&before, ArchiveFormat::JSON, Compression::Plain)
            .expect("Should not fail to serialize");
        assert!(bytes.len() > 0);
        
        let after = archive_decode(bytes, ArchiveFormat::JSON, Compression::Plain).expect("Should not fail to deserialize");
        assert_eq!(after, before);
    }
}
