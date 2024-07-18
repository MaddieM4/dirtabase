// Import/Export format that doesn't require DB API
use std::path::Path;
use std::io::{Read,Result,Cursor};

pub trait Sink where Self: Sized {
    // Required methods
    fn send_dir(self, path: impl AsRef<Path>) -> Result<Self>;
    fn send_file(self, path: impl AsRef<Path>, r: impl Read) -> Result<Self>;
    fn finalize(self) -> Result<()>;
}

pub struct DebugSink<'a>(&'a mut String);
impl Sink for DebugSink<'_> {
    fn send_dir(self, path: impl AsRef<Path>) -> Result<Self> {
        self.0.push_str(&format!("DIR {}\n", path.as_ref().to_string_lossy()));
        Ok(self)
    }
    fn send_file(self, path: impl AsRef<Path>, mut r: impl Read) -> Result<Self> {
        let mut buf: Vec<u8> = vec![];
        r.read_to_end(&mut buf)?;
        self.0.push_str(&format!("FILE {}\n", path.as_ref().to_string_lossy()));
        self.0.push_str(&format!("  Length: {}\n", buf.len()));
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

    #[test]
    fn builder_api() -> Result<()> {
        let mut s = String::new();
        DebugSink(&mut s)
            .send_dir("/a/deep/directory")?
            .send_file("/some/file.txt", Cursor::new("contents"))?
            .finalize()?;

        assert_eq!(&s, indoc! {"
            DIR /a/deep/directory
            FILE /some/file.txt
              Length: 8
        "});
        Ok(())
    }
}
