use serde::{Deserialize,Serialize};
use crate::digest::Digest;

#[derive(PartialEq,Debug,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Format {
    File,
    JSON,
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Compression {
    Plain,
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
pub struct Attr(String,String);
impl Attr {
    fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self(name.as_ref().into(), value.as_ref().into())
    }
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
pub struct ArchiveEntry {
    path: String,
    format: Format,
    compression: Compression,
    digest: Digest,
    attrs: Vec<Attr>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_roundtrip() -> serde_json::Result<()> {
        let archive: Vec<ArchiveEntry> = vec![
            ArchiveEntry {
                path: "foo/bar.txt".into(),
                format: Format::File,
                compression: Compression::Plain,
                digest: "some text".into(),
                attrs: vec![
                  Attr::new("unix_owner", "1000"),
                  Attr::new("unix_group", "1000"),
                  Attr::new("unix_flags", "0x777"),
                  Attr::new("frob_value", "absolutely frobnicated"),
                ]
            }
        ];

        let text: String = serde_json::to_string(&archive)?;
        assert_eq!(&text, r#"[{"path":"foo/bar.txt","format":"file","compression":"plain","digest":[185,79,111,18,92,121,227,165,255,170,130,111,88,76,16,213,42,218,102,158,103,98,5,27,130,107,85,119,109,5,174,210],"attrs":[["unix_owner","1000"],["unix_group","1000"],["unix_flags","0x777"],["frob_value","absolutely frobnicated"]]}]"#);

        let deserialized: Vec<ArchiveEntry> = serde_json::from_str(&text)?;
        assert_eq!(&deserialized, &archive);

        Ok(())
    }
}
