//! API for storing and retrieving potentially large files by digest.

pub mod simple;
use simple::SimpleStorage;

pub fn new<P>(path: P) -> std::io::Result<SimpleStorage<P>> where P: AsRef<std::path::Path> {
    SimpleStorage::new(path)
}

pub fn new_from_tempdir() -> std::io::Result<SimpleStorage<tempfile::TempDir>> {
    SimpleStorage::new(tempfile::tempdir()?)
}
