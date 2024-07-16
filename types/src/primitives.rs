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

#[derive(PartialEq,Debug,Clone,Copy,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Format {
    File,
    JSON,
}

#[derive(PartialEq,Debug,Clone,Copy,Serialize,Deserialize)]
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

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct Attr(String,String);
impl Attr {
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self(name.as_ref().into(), value.as_ref().into())
    }
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct ArchiveEntry {
    pub path: String,
    pub spec: Spec,
    pub attrs: Vec<Attr>,
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct Archive {
    pub format: Format,
    pub compression: Compression,
    pub entries: Vec<ArchiveEntry>,
}

impl From<&Archive> for Resource {
    fn from(archive: &Archive) -> Self {
        archive.to_buffer().into()
    }
}

impl Archive {
    pub fn set(&mut self, entry: &ArchiveEntry) {
        self.entries.retain(|e| e.path != entry.path);
        self.entries.push(entry.clone());
    }

    pub fn to_buffer(&self) -> Buffer {
        assert!(self.format == Format::JSON);
        assert!(self.compression == Compression::Plain);
        serde_json::to_vec(&self.entries).unwrap()
    }

    pub fn from_buffer(format: Format, compression: Compression, buf: &Buffer) -> Self {
        assert!(format == Format::JSON);
        assert!(compression == Compression::Plain);
        let entries: Vec<ArchiveEntry> = serde_json::from_slice(buf).unwrap();
        Archive {
            format: format,
            compression: compression,
            entries: entries,
        }
    }
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
        let archive = Archive {
            format: Format::JSON,
            compression: Compression::Plain,
            entries: vec![
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
            ],
        };

        let buf = archive.to_buffer();
        assert_eq!(
            String::from_utf8(buf.clone()).unwrap(),
            r#"[{"path":"foo/bar.txt","spec":{"format":"file","compression":"plain","digest":[185,79,111,18,92,121,227,165,255,170,130,111,88,76,16,213,42,218,102,158,103,98,5,27,130,107,85,119,109,5,174,210]},"attrs":[["unix_owner","1000"],["unix_group","1000"],["unix_flags","0x777"],["frob_value","absolutely frobnicated"]]}]"#);

        let deserialized = Archive::from_buffer(archive.format, archive.compression, &buf);
        assert_eq!(&deserialized, &archive);

        Ok(())
    }

    #[test]
    fn archive_set() {
        let mut archive = Archive {
            format: Format::JSON,
            compression: Compression::Plain,
            entries: vec![],
        };

        let spec = Spec {
            format: Format::File,
            compression: Compression::Plain,
            digest: "some contents".into(),
        };
        let entry1 = ArchiveEntry {
            path: "same/path".into(),
            spec: spec.clone(),
            attrs: vec![Attr::new("entry","1")],
        };
        let entry2 = ArchiveEntry {
            path: "different/path".into(),
            spec: spec.clone(),
            attrs: vec![Attr::new("entry","2")],
        };
        let entry3 = ArchiveEntry {
            path: "same/path".into(),
            spec: spec.clone(),
            attrs: vec![Attr::new("entry","3")],
        };

        archive.set(&entry1);
        assert_eq!(archive.entries, vec![entry1.clone()]);

        archive.set(&entry2);
        assert_eq!(archive.entries, vec![entry1.clone(), entry2.clone()]);

        // Override existing path
        archive.set(&entry3);
        assert_eq!(archive.entries, vec![entry2.clone(), entry3.clone()]);
    }
}
