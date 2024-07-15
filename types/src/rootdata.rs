use crate::digest::Digest;
use crate::archive::{Format,Compression};

pub struct RootData {
    format: Format,
    compression: Compression,
    digest: Digest,
}
