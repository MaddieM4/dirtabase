use crate::digest::Digest;
use serde::{Deserialize,Serialize};

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

#[derive(PartialEq,Debug,Clone,Copy,Serialize)]
pub struct Spec(Format,Compression,Digest);

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn serialize_spec() {
        let spec = Spec(Format::File, Compression::Plain, Digest::from("foo"));
        let txt = to_string(&spec).expect("Serialized without errors");
        assert_eq!(txt, r#"["file","plain","2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"]"#);
    }
}
