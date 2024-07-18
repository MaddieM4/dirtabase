// Import/Export format that doesn't require DB API
use std::path::Path;
use std::io::{Read,Result};
use crate::primitives::Attrs;

pub trait Sink where Self: Sized {
    // Required methods
    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self>;
    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, r: impl Read) -> Result<Self>;
    fn finalize(self) -> Result<()>;
}

pub struct DebugSink<'a>(&'a mut String);
impl Sink for DebugSink<'_> {
    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self> {
        self.0.push_str(&format!("DIR {}\n", path.as_ref().to_string_lossy()));
        for attr in attrs {
            self.0.push_str(&format!("  {}: {}\n", attr.name(), attr.value()));
        }
        Ok(self)
    }
    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, mut r: impl Read) -> Result<Self> {
        let mut buf: Vec<u8> = vec![];
        r.read_to_end(&mut buf)?;
        self.0.push_str(&format!("FILE {}\n", path.as_ref().to_string_lossy()));
        self.0.push_str(&format!("  Length: {}\n", buf.len()));
        for attr in attrs {
            self.0.push_str(&format!("  {}: {}\n", attr.name(), attr.value()));
        }
        Ok(self)
    }
    fn finalize(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use std::io::Cursor;
    use crate::primitives::Attr;

    #[test]
    fn builder_api() -> Result<()> {
        let mut s = String::new();
        DebugSink(&mut s)
            .send_dir("/a/deep/directory", vec![])?
            .send_file("/some/file.txt", vec![], Cursor::new("contents"))?
            .send_dir("/dir/with/attrs", vec![
                Attr::new("unix_owner", "1000"),
                Attr::new("unix_group", "2000"),
            ])?
            .send_file("/file/with/attrs", vec![
                Attr::new("A", "a"),
                Attr::new("B", "b"),
                Attr::new("C", "c"),
            ], Cursor::new("And also longer contents"))?
            .finalize()?;

        assert_eq!(&s, indoc! {"
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
        "});
        Ok(())
    }
}
