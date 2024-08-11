use crate::archive::core::{ArchiveFormat, Compression};

#[derive(Copy, Clone)]
pub struct Settings(ArchiveFormat, Compression);
impl Settings {
    pub fn f(&self) -> ArchiveFormat {
        self.0
    }
    pub fn c(&self) -> Compression {
        self.1
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self(ArchiveFormat::JSON, Compression::Plain)
    }
}
