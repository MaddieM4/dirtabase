//! Logic for fetching and storing data from the disk.

use super::types::*;
use crate::attr::Attrs;
use std::fs::Metadata;
use std::io::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn recursive_accumulate(cur: &Path, output: &mut Vec<(PathBuf, Metadata)>) -> Result<()> {
    if cur.is_dir() {
        for entry in std::fs::read_dir(cur)? {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;

            if meta.is_dir() {
                recursive_accumulate(&path, output)?;
            }
            output.push((path, meta));
        }
    }
    Ok(())
}

impl From<Metadata> for Attrs {
    fn from(meta: Metadata) -> Attrs {
        Attrs::new().append("UNIX_MODE", meta.permissions().mode().to_string())
    }
}

impl Ark<PathBuf> {
    /// Fetch metadata for a directory into memory.
    ///
    /// This isn't a parallel process, but it's fast, and allows subsequent
    /// steps to load in a high-performance parallel way.
    pub fn scan(base: impl AsRef<Path>) -> Result<Self> {
        let mut acc: Vec<(PathBuf, Metadata)> = vec![];
        recursive_accumulate(base.as_ref(), &mut acc)?;

        Ok(Self::from(
            acc.into_iter()
                .map(|(pb, meta)| {
                    let p = pb
                        .strip_prefix(&base)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let c = if meta.is_dir() {
                        Contents::Dir
                    } else {
                        Contents::File(pb)
                    };
                    let a: Attrs = meta.into();
                    (p, a, c)
                })
                .collect::<Vec<(IPR, Attrs, Contents<PathBuf>)>>(),
        ))
    }

    /// Fetch file contents from disk into memory.
    ///
    /// Be warned that this may be a very bad idea if the directory is larger
    /// than you have RAM for.
    pub fn read(self) -> Result<Ark<Vec<u8>>> {
        let (paths, attrs, contents) = self.decompose();
        let contents: Result<Vec<Vec<u8>>> =
            contents.into_iter().map(|pb| std::fs::read(&pb)).collect();
        Ok(Ark::compose(paths, attrs, contents?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn scan() -> Result<()> {
        let ark = Ark::scan("./fixture")?;

        /*
        fixture
        ├── dir1
        │   └── dir2
        │       └── nested.txt
        └── file_at_root.txt
        */

        assert_eq!(
            ark.paths(),
            &vec![
                "dir1/dir2/nested.txt",
                "file_at_root.txt",
                "dir1",
                "dir1/dir2",
            ]
        );
        assert_eq!(
            ark.attrs(),
            &vec![
                at! { UNIX_MODE => "33204" },
                at! { UNIX_MODE => "33204" },
                at! { UNIX_MODE => "16893" },
                at! { UNIX_MODE => "16893" },
            ]
        );
        assert_eq!(ark.contents().len(), 2);
        Ok(())
    }

    #[test]
    fn read() -> Result<()> {
        let ark = Ark::scan("./fixture")?.read()?;
        assert_eq!(
            ark.paths(),
            &vec![
                "dir1/dir2/nested.txt",
                "file_at_root.txt",
                "dir1",
                "dir1/dir2",
            ]
        );
        assert_eq!(
            ark.attrs(),
            &vec![
                at! { UNIX_MODE => "33204" },
                at! { UNIX_MODE => "33204" },
                at! { UNIX_MODE => "16893" },
                at! { UNIX_MODE => "16893" },
            ]
        );
        assert_eq!(
            ark.contents(),
            &vec![
                "A file nested under multiple directories\n".into(),
                "Here are some file contents, teehee!\n".into(),
            ] as &Vec<Vec<u8>>
        );
        Ok(())
    }
}
