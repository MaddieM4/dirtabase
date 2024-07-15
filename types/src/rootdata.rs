use crate::digest::Digest;
use crate::archive::{Format,Compression};

#[derive(Debug,PartialEq,Clone)]
pub struct Spec {
    pub format: Format,
    pub compression: Compression,
    pub digest: Digest,
}

pub type RootData = Option<Spec>;
