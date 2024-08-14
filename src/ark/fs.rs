//! Logic for fetching and storing data from the disk.

use super::types::*;
use crate::attr::Attrs;
use crate::digest::Digest;
use std::fs::Metadata;
use std::io::Result;
use std::iter::zip;
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
    pub fn scan_disk(base: impl AsRef<Path>) -> Result<Self> {
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

    /// Import files into an on-disk database.
    pub fn import(&self, store_path: &Path) -> Result<Ark<Digest>> {
        let (paths, attrs, src_pathbufs) = (self.paths(), self.attrs(), self.contents());
        let store_path = init_store(store_path.as_ref(), ["tmp", "cas"])?;

        // Copy files to temp location
        let dest = tempfile::tempdir_in(store_path.join("tmp"))?;
        let temps: Result<Vec<PathBuf>> = src_pathbufs
            .iter()
            .enumerate()
            .map(|(n, file_src)| {
                let file_dest = dest.as_ref().join(n.to_string());
                std::fs::copy(file_src, &file_dest)?;
                Ok(file_dest.to_path_buf())
            })
            .collect();
        let temps = temps?;

        let digests = hash_files(&temps)?;
        for (temp, digest) in zip(temps, &digests) {
            std::fs::rename(temp, store_path.join("cas").join(digest.to_hex()))?;
        }

        Ok(Ark::compose(paths.clone(), attrs.clone(), digests))
    }
}

impl Ark<Vec<u8>> {
    /// Import files into an on-disk database.
    pub fn import(&self, store_path: impl AsRef<Path>) -> Result<Ark<Digest>> {
        let (paths, attrs, contents) = (self.paths(), self.attrs(), self.contents());
        let store_path = init_store(store_path.as_ref(), ["tmp", "cas"])?;

        // Write files to temp location
        let dest = tempfile::tempdir_in(store_path.join("tmp"))?;
        let temps: Result<Vec<PathBuf>> = contents
            .iter()
            .enumerate()
            .map(|(n, content)| {
                let file_dest = dest.as_ref().join(n.to_string());
                std::fs::write(&file_dest, content)?;
                Ok(file_dest.to_path_buf())
            })
            .collect();
        let temps = temps?;

        let digests = hash_files(&temps)?;
        for (temp, digest) in zip(temps, &digests) {
            std::fs::rename(temp, store_path.join("cas").join(digest.to_hex()))?;
        }

        Ok(Ark::compose(paths.clone(), attrs.clone(), digests))
    }
}

fn init_store<'a, const N: usize>(root: &'a Path, sections: [&str; N]) -> Result<&'a Path> {
    for section in sections {
        let p = root.join(section);
        if !p.exists() {
            std::fs::create_dir(p)?;
        }
    }
    Ok(root)
}

fn hash_files(paths: &Vec<PathBuf>) -> Result<Vec<Digest>> {
    // TODO: Parallelize with Rayon, compare speed
    paths
        .iter()
        .map(|pb| {
            let f = std::fs::File::open(pb)?;
            let mmap = unsafe { memmap::Mmap::map(&f)? };
            Ok(Digest::from(mmap.as_ref()))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn scan_disk() -> Result<()> {
        let ark = Ark::scan_disk("./fixture")?;

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
        let ark = Ark::scan_disk("./fixture")?.read()?;
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

    #[test]
    fn import() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let ark = Ark::scan_disk("fixture")?.import(tmp.as_ref())?;

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

        let expected_text = "A file nested under multiple directories\n";
        let d = ark.contents()[0];
        let p = tmp.as_ref().join("cas").join(d.to_hex());

        assert_eq!(d, Digest::from(expected_text));
        assert_eq!(std::fs::read_to_string(p)?, expected_text.to_owned());

        // Get same results if we start from in-mem copy
        assert_eq!(
            Ark::scan_disk("fixture")?.read()?.import(tmp.as_ref())?,
            ark
        );

        Ok(())
    }
}
