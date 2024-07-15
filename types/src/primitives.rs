use crate::digest::Digest;
use serde::{Deserialize,Serialize};

pub type Buffer = Vec<u8>;

#[derive(Debug,PartialEq)]
pub struct Resource {
    pub digest: Digest,
    pub body: Buffer,
}
impl<T> From<T> for Resource where T: AsRef<[u8]> {
    fn from(item: T) -> Self {
        Resource {
            digest: Digest::from(&item),
            body: item.as_ref().into(),
        }
    }
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Format {
    File,
    JSON,
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Compression {
    Plain,
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct Spec {
    pub format: Format,
    pub compression: Compression,
    pub digest: Digest,
}
pub type RootData = Option<Spec>;

#[derive(PartialEq,Debug,Serialize,Deserialize)]
pub struct Attr(String,String);
impl Attr {
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self(name.as_ref().into(), value.as_ref().into())
    }
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
pub struct ArchiveEntry {
    path: String,
    spec: Spec,
    attrs: Vec<Attr>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resource_from_static_str() {
        let s = "Hello world!";
        let r = Resource::from(s);
        assert_eq!(r.body, Vec::<u8>::from(s));
        assert_eq!(r.digest.to_hex(), Digest::from(s).to_hex());
    }

    #[test]
    fn resource_from_string() {
        let s = "Hello world!".to_string();
        let sc: Buffer = s.clone().into();
        let r = Resource::from(s); // Moves s
        assert_eq!(r.body, sc);
        assert_eq!(r.digest.to_hex(), Digest::from(&sc).to_hex());
    }

    #[test]
    fn serde_archive() -> serde_json::Result<()> {
        let archive: Vec<ArchiveEntry> = vec![
            ArchiveEntry {
                path: "foo/bar.txt".into(),
                spec: Spec {
                  format: Format::File,
                  compression: Compression::Plain,
                  digest: "some text".into(),
                },
                attrs: vec![
                  Attr::new("unix_owner", "1000"),
                  Attr::new("unix_group", "1000"),
                  Attr::new("unix_flags", "0x777"),
                  Attr::new("frob_value", "absolutely frobnicated"),
                ]
            }
        ];

        let text: String = serde_json::to_string(&archive)?;
        assert_eq!(&text, r#"[{"path":"foo/bar.txt","spec":{"format":"file","compression":"plain","digest":[185,79,111,18,92,121,227,165,255,170,130,111,88,76,16,213,42,218,102,158,103,98,5,27,130,107,85,119,109,5,174,210]},"attrs":[["unix_owner","1000"],["unix_group","1000"],["unix_flags","0x777"],["frob_value","absolutely frobnicated"]]}]"#);

        let deserialized: Vec<ArchiveEntry> = serde_json::from_str(&text)?;
        assert_eq!(&deserialized, &archive);

        Ok(())
    }
}
