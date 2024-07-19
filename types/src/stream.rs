// Import/Export format that doesn't require DB API
use crate::primitives::Attrs;
use std::io::{Read, Result};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

pub trait Sink
where
    Self: Sized,
{
    // Required methods
    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self>;
    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, r: impl Read) -> Result<Self>;
    fn finalize(self) -> Result<()>;
}

pub struct DebugSink<'a>(&'a mut String);
impl Sink for DebugSink<'_> {
    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self> {
        self.0
            .push_str(&format!("DIR {}\n", path.as_ref().to_string_lossy()));
        for attr in attrs {
            self.0
                .push_str(&format!("  {}: {}\n", attr.name(), attr.value()));
        }
        Ok(self)
    }
    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, mut r: impl Read) -> Result<Self> {
        let mut buf: Vec<u8> = vec![];
        r.read_to_end(&mut buf)?;
        self.0
            .push_str(&format!("FILE {}\n", path.as_ref().to_string_lossy()));
        self.0.push_str(&format!("  Length: {}\n", buf.len()));
        for attr in attrs {
            self.0
                .push_str(&format!("  {}: {}\n", attr.name(), attr.value()));
        }
        Ok(self)
    }
    fn finalize(self) -> Result<()> {
        Ok(())
    }
}

fn normal_join(base: impl AsRef<Path>, rel: impl AsRef<Path>) -> PathBuf {
    let rel = rel.as_ref();
    base.as_ref().join(match rel.is_absolute() {
        true => rel.strip_prefix("/").expect("Could not de-absolute rel path"),
        false => rel,
    })
}

// Creates a directory within a real filesystem.
// Builds fresh in a temp directory.
// Finalizing does an atomic rename of temp dir to target location.
pub struct OsdirSink {
    tmp: TempDir,
    dest: PathBuf,
}
impl OsdirSink {
    pub fn new(dest: impl AsRef<Path>) -> Self {
        Self {
            tmp: TempDir::new("dirtabase").expect("Could not allocate tempdir"),
            dest: dest.as_ref().into(),
        }
    }

    fn normalize(&self, path: impl AsRef<Path>) -> PathBuf {
        normal_join(self.tmp.path(), path)
    }
}
impl Sink for OsdirSink {
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
        std::fs::rename(src, self.dest)?;
        Ok(())
    }
}

fn _osdir_visit<S>(base: &Path, rel: &Path, mut sink: S) -> Result<S> where S: Sink {
    let real_path = normal_join(base, rel);
    for entry in std::fs::read_dir(real_path)? {
        let dir = entry?;
        let virt_path = rel.join(&dir.file_name());
        let file_type = dir.file_type()?;
        if file_type.is_dir() {
            sink = sink.send_dir(&virt_path, vec![])?;
            sink = _osdir_visit(&base, &virt_path, sink)?;
        } else if file_type.is_file() {
            let reader = std::fs::File::open(&dir.path())?;
            sink = sink.send_file(virt_path, vec![], reader)?
        }
    }
    Ok(sink)
}

pub fn osdir_src(base: impl AsRef<Path>, sink: impl Sink) -> Result<()> {
    _osdir_visit(base.as_ref(), Path::new("/"), sink)?.finalize()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::primitives::Attr;
    use indoc::indoc;
    use std::io::Cursor;

    #[test]
    fn builder_api() -> Result<()> {
        let mut s = String::new();
        DebugSink(&mut s)
            .send_dir("/a/deep/directory", vec![])?
            .send_file("/some/file.txt", vec![], Cursor::new("contents"))?
            .send_dir(
                "/dir/with/attrs",
                vec![
                    Attr::new("unix_owner", "1000"),
                    Attr::new("unix_group", "2000"),
                ],
            )?
            .send_file(
                "/file/with/attrs",
                vec![
                    Attr::new("A", "a"),
                    Attr::new("B", "b"),
                    Attr::new("C", "c"),
                ],
                Cursor::new("And also longer contents"),
            )?
            .finalize()?;

        assert_eq!(
            &s,
            indoc! {"
            DIR /a/deep/directory
            FILE /some/file.txt
              Length: 8
            DIR /dir/with/attrs
              unix_owner: 1000
              unix_group: 2000
            FILE /file/with/attrs
              Length: 24
              A: a
              B: b
              C: c
        "}
        );
        Ok(())
    }

    #[test]
    fn osdir_sink() -> Result<()> {
        let dest = Path::new("/tmp/test_osdir_sink");
        if dest.exists() {
            std::fs::remove_dir_all(dest)?;
        }

        OsdirSink::new(dest)
            .send_dir("/some/place", vec![])?
            .send_file("/hello/world.txt", vec![], Cursor::new("Some text"))?
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
    fn _osdir_src() -> Result<()> {
        let mut s = String::new();
        osdir_src("./fixture", DebugSink(&mut s))?;
        assert_eq!(
            &s,
            indoc! {"
            FILE /file_at_root.txt
              Length: 37
            DIR /dir1
            DIR /dir1/dir2
            FILE /dir1/dir2/nested.txt
              Length: 41
        "}
        );
        Ok(())
    }
}
