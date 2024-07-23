pub use crate::digest::Digest;
pub use crate::attr::Attrs;
use serde::{Deserialize,Serialize};
use std::path::PathBuf;

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

#[derive(PartialEq,Debug,Clone,Copy,Serialize,Deserialize)]
pub struct Triad(pub Format,pub Compression,pub Digest);

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct Entry(pub PathBuf,pub Triad,pub Attrs);

pub type Archive = Vec<Entry>;

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{to_string,from_str};

    #[test]
    fn triad_serialize() {
        let triad = Triad(Format::File, Compression::Plain, Digest::from("foo"));
        let txt = to_string(&triad).expect("Serialized without errors");
        assert_eq!(txt, r#"["file","plain","2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"]"#);
    }

    #[test]
    fn triad_deserialize() {
        let txt = r#"["file","plain","2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"]"#;
        let triad: Triad = from_str(&txt).expect("Deserialized without errors");
        assert_eq!(triad, Triad(Format::File, Compression::Plain, Digest::from("foo")));
    }

}
