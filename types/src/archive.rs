use serde::{Deserialize,Serialize};

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
