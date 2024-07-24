//! Import and export from real directories on your operating system.
//!
//! ```
//! use dirtabase::stream::osdir::{source,sink};
//! source("./fixture", sink("./copy_of_fixture")).expect("copied!");
//! # std::fs::remove_dir_all("./copy_of_fixture")?;
//! # Ok::<(), std::io::Error>(())
//! ```

use crate::attr::*;
use crate::stream::core::Sink;
use std::io::{Read, Result};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

/// Read from a real directory and emit to the given sink.
///
/// ```
/// use dirtabase::stream::osdir::source;
/// use dirtabase::stream::debug::sink; // Debug sink
/// use indoc::indoc;
///
/// let mut s = String::new();
/// source("./fixture", sink(&mut s))?;
/// assert_eq!(&s, indoc! {"
///   FILE /file_at_root.txt
///     Length: 37
///   DIR /dir1
///   DIR /dir1/dir2
///   FILE /dir1/dir2/nested.txt
///     Length: 41
/// "});
///
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn source<S>(base: impl AsRef<Path>, sink: S) -> Result<S::Receipt> where S: Sink {
    visit(base.as_ref(), Path::new("/"), sink)?.finalize()
}

/// Creates a directory within a real filesystem.
///
/// Builds fresh in a temp directory. Finalizing does an atomic rename of temp
/// dir to target location. If the destination exists, it is deleted before
/// rename.
///
/// ```
/// use dirtabase::attr::Attrs;
/// use dirtabase::stream::core::Sink;
/// use dirtabase::stream::osdir::sink;
/// use std::io::Cursor;
///
/// sink("./.temp")
///     .send_file(
///         "some/dir/hello.txt",
///         Attrs::new(),
///         Cursor::new("The file contents"))?
///     .finalize()?;
/// # std::fs::remove_dir_all("./.temp")?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn sink(dest: impl AsRef<Path>) -> OsdirSink {
    OsdirSink::new(dest)
}

fn normal_join(base: impl AsRef<Path>, rel: impl AsRef<Path>) -> PathBuf {
    let rel = rel.as_ref();
    base.as_ref().join(match rel.is_absolute() {
        true => rel
            .strip_prefix("/")
            .expect("Could not de-absolute rel path"),
        false => rel,
    })
}

/// Implementation of sink().
pub struct OsdirSink {
    tmp: TempDir,
    dest: PathBuf,
}
impl OsdirSink {
    pub fn new(dest: impl AsRef<Path>) -> Self {
        let pb: PathBuf = dest.as_ref().into();
        let parent = pb
            .parent()
            .expect("Could not get parent of osdir::sink destination");
        let tmp = TempDir::new_in(parent, ".dirtabase").expect("Could not allocate tempdir");
        Self { tmp: tmp, dest: pb }
    }

    fn normalize(&self, path: impl AsRef<Path>) -> PathBuf {
        normal_join(self.tmp.path(), path)
    }
}
impl Sink for OsdirSink {
    type Receipt = ();

    fn send_dir(self, path: impl AsRef<Path>, _attrs: Attrs) -> Result<Self> {
        // TODO: use attrs
        let path = self.normalize(path.as_ref());
        std::fs::create_dir_all(path)?;
        Ok(self)
    }
    fn send_file(self, path: impl AsRef<Path>, _attrs: Attrs, mut r: impl Read) -> Result<Self> {
        let path = path.as_ref();
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::PermissionDenied))?;
        let (path, parent) = (self.normalize(path), self.normalize(parent));

        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
        // TODO: use attrs
        let mut w = std::fs::File::create(path)?;
        std::io::copy(&mut r, &mut w)?;
        Ok(self)
    }
    fn finalize(self) -> Result<()> {
        let src = self.tmp.into_path();
        if self.dest.exists() {
            std::fs::remove_dir_all(&self.dest)?;
        }
        std::fs::rename(src, self.dest)?;
        Ok(())
    }
}

fn visit<S>(base: &Path, rel: &Path, mut sink: S) -> Result<S>
where
    S: Sink,
{
    let real_path = normal_join(base, rel);
    for entry in std::fs::read_dir(real_path)? {
        let dir = entry?;
        let virt_path = rel.join(&dir.file_name());
        let file_type = dir.file_type()?;
        if file_type.is_dir() {
            sink = sink.send_dir(&virt_path, Attrs::new())?;
            sink = visit(&base, &virt_path, sink)?;
        } else if file_type.is_file() {
            let reader = std::fs::File::open(&dir.path())?;
            sink = sink.send_file(virt_path, Attrs::new(), reader)?
        }
    }
    Ok(sink)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn osdir_sink() -> Result<()> {
        let tmp_dest = TempDir::new("dirtabase")?;
        let dest = tmp_dest.path();

        OsdirSink::new(dest)
            .send_dir("/some/place", Attrs::new())?
            .send_file("/hello/world.txt", Attrs::new(), Cursor::new("Some text"))?
            .finalize()?;

        assert!(dest.exists());
        assert!(dest.join("some/place").exists());
        assert!(dest.join("hello/world.txt").exists());
        assert_eq!(
            std::fs::read(dest.join("hello/world.txt"))?,
            Vec::<u8>::from("Some text")
        );
        Ok(())
    }

    #[test]
    fn osdir_round_trip() -> Result<()> {
        let tmp_dest = TempDir::new("dirtabase")?;
        let dest = tmp_dest.path();

        source("./fixture", OsdirSink::new(dest))?;
        assert!(dest.exists());
        assert!(dest.join("dir1/dir2/nested.txt").exists());
        assert_eq!(
            std::fs::read(dest.join("file_at_root.txt"))?,
            std::fs::read("./fixture/file_at_root.txt")?,
        );
        Ok(())
    }
}
