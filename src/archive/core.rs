pub use crate::attr::Attrs;
pub use crate::digest::Digest;

pub use serde::{Deserialize, Serialize};
pub use std::path::{Path, PathBuf};

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    JSON,
}
impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::JSON => "json",
        })
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    Plain,
}
impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Plain => "plain",
        })
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag="t", content="c", rename_all="lowercase")]
pub enum Entry {
    Dir {
        path: PathBuf,
        attrs: Attrs,
    },
    File {
        path: PathBuf,
        attrs: Attrs,
        compression: Compression,
        digest: Digest,
    },
    // TODO: Sub-archives
    // Archive {
    //  path: PathBuf, attrs: Attrs,
    //  format: ArchiveFormat, compression: Compression,
    //  digest: Digest,
    // }
}

pub type Archive = Vec<Entry>;

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriadFormat {
    File,
    Archive(ArchiveFormat),
}
impl std::fmt::Display for TriadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::File => "file".to_owned(),
            Self::Archive(af) => af.to_string(),
        })
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Triad(pub TriadFormat, pub Compression, pub Digest);
impl std::fmt::Display for Triad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.0, self.1, self.2.to_hex())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn triad_serialize() {
        let triad = Triad(TriadFormat::File, Compression::Plain, Digest::from("foo"));
        let txt = to_string(&triad).expect("Serialized without errors");
        assert_eq!(
            txt,
            r#"["file","plain","2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"]"#
        );
    }

    #[test]
    fn triad_deserialize() {
        let txt = r#"["file","plain","2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"]"#;
        let triad: Triad = from_str(&txt).expect("Deserialized without errors");
        assert_eq!(
            triad,
            Triad(TriadFormat::File, Compression::Plain, Digest::from("foo"))
        );
    }

    #[test]
    fn triad_display() {
        let triad = Triad(TriadFormat::File, Compression::Plain, Digest::from("foo"));
        assert_eq!(triad.to_string(), "file-plain-2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae");
        let triad = Triad(TriadFormat::Archive(ArchiveFormat::JSON), Compression::Plain, Digest::from("foo"));
        assert_eq!(triad.to_string(), "json-plain-2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae");
    }
}
