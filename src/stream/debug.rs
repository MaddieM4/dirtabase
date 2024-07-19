//! Simple sink and source for diagnostic purposes.
//!
//! ```
//! use dirtabase::stream::debug::{source,sink};
//! use indoc::indoc;
//!
//! let mut s = String::new();
//! source(sink(&mut s)).expect("Really can't fail");
//! assert_eq!(&s, indoc! {"
//!   DIR /a/directory
//!     Foo: Bar
//!   FILE /some/dir/hello.txt
//!     Length: 17
//!     AnotherAttr: for example purposes
//! "});
//! ```

use crate::attr::*;
use crate::stream::core::Sink;
use std::io::{Read, Result};
use std::path::Path;
use std::io::Cursor;

/// Send a standard series of directories and files.
///
/// Used for various tests (for example, this module's docs!)
pub fn source(s: impl Sink) -> Result<()> {
    s.send_dir("/a/directory", Attrs::new().set("Foo", "Bar"))?
        .send_file(
            "/some/dir/hello.txt",
            Attrs::new().set("AnotherAttr", "for example purposes"),
            Cursor::new("The file contents"),
        )?
        .finalize()
}

/// Logs all writes to a mutable String buffer.
///
/// ```
/// use dirtabase::attr::Attrs;
/// use dirtabase::stream::core::Sink;
/// use dirtabase::stream::debug::sink;
/// use std::io::Cursor;
/// use indoc::indoc;
///
/// let mut s = String::new();
/// sink(&mut s)
///     .send_dir("/a/directory", Attrs::new().set("Foo","Bar"))?
///     .send_file(
///         "/some/dir/hello.txt",
///         Attrs::new().set("AnotherAttr", "for example purposes"),
///         Cursor::new("The file contents"))?
///     .finalize()?;
///
/// assert_eq!(&s, indoc! {"
///   DIR /a/directory
///     Foo: Bar
///   FILE /some/dir/hello.txt
///     Length: 17
///     AnotherAttr: for example purposes
/// "});
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn sink(s: &mut String) -> DebugSink {
    DebugSink(s)
}

/// Implementation of sink().
pub struct DebugSink<'a>(&'a mut String);

impl DebugSink<'_> {
    fn write_line(self, line: &str) -> Self {
        self.0.push_str(line);
        self
    }

    fn write_head(self, item_type: &'static str, path: impl AsRef<Path>) -> Self {
        self.write_line(&format!(
            "{} {}\n",
            item_type,
            path.as_ref().to_string_lossy()
        ))
    }

    fn write_attrs(self, attrs: Attrs) -> Self {
        for attr in attrs.items() {
            let text = format!("  {}: {}\n", attr.name(), attr.value());
            self.0.push_str(&text);
        }
        self
    }
}

impl Sink for DebugSink<'_> {
    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self> {
        Ok(self.write_head("DIR", path).write_attrs(attrs))
    }
    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, mut r: impl Read) -> Result<Self> {
        let size = std::io::copy(&mut r, &mut std::io::empty())?;
        Ok(self
            .write_head("FILE", path)
            .write_line(&format!("  Length: {}\n", size))
            .write_attrs(attrs))
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

    #[test]
    fn builder_api() -> Result<()> {
        let mut s = String::new();
        DebugSink(&mut s)
            .send_dir("/a/deep/directory", Attrs::new())?
            .send_file("/some/file.txt", Attrs::new(), Cursor::new("contents"))?
            .send_dir(
                "/dir/with/attrs",
                Attrs::new()
                    .append("unix_owner", "1000")
                    .append("unix_group", "2000"),
            )?
            .send_file(
                "/file/with/attrs",
                Attrs::new()
                    .append("A", "a")
                    .append("B", "b")
                    .append("C", "c"),
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
}
