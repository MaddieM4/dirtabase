use super::save::Save;
use super::types::*;
use crate::db::DB;
use crate::digest::Digest;
use std::io::Result;
use std::iter::zip;
use std::path::PathBuf;

fn hash_file(pb: &PathBuf) -> Result<Digest> {
    let f = std::fs::File::open(pb)?;

    if f.metadata()?.len() == 0 {
        // Unfortunately it's an error to map an empty file.
        // It's deeply obnoxious to need a second metadata call here.
        // Maybe a solution will eventually present itself, or perhaps when
        // actually benched, the cost of this op is trivial. Hard to say!
        return Ok(Digest::from(""));
    }

    let mmap = unsafe { memmap::Mmap::map(&f)? };
    Ok(Digest::from(mmap.as_ref()))
}

fn hash_files(paths: &Vec<PathBuf>) -> Result<Vec<Digest>> {
    // TODO: Parallelize with Rayon, compare speed
    paths.iter().map(|pb| hash_file(pb)).collect()
}

pub trait Import {
    /// Import files into an on-disk database.
    fn import_files(&self, db: &DB) -> Result<Ark<Digest>>;

    /// Import files _and_ serialized self into DB.
    fn import(&self, db: &DB) -> Result<Digest> {
        self.import_files(db)?.save(db)
    }
}

impl Import for Ark<PathBuf> {
    fn import_files(&self, db: &DB) -> Result<Ark<Digest>> {
        let (paths, attrs, src_pathbufs) = (self.paths(), self.attrs(), self.contents());

        // Copy files to temp location
        let dest = tempfile::tempdir_in(db.join("tmp"))?;
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
            std::fs::rename(temp, db.join("cas").join(digest.to_hex()))?;
        }

        Ok(Ark::compose(paths.clone(), attrs.clone(), digests))
    }
}

impl Import for Ark<Vec<u8>> {
    fn import_files(&self, db: &DB) -> Result<Ark<Digest>> {
        let (paths, attrs, contents) = (self.paths(), self.attrs(), self.contents());

        // Write files to temp location
        let dest = tempfile::tempdir_in(db.join("tmp"))?;
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
            std::fs::rename(temp, db.join("cas").join(digest.to_hex()))?;
        }

        Ok(Ark::compose(paths.clone(), attrs.clone(), digests))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;
    use crate::attr::Attrs;

    #[test]
    fn import_files() -> Result<()> {
        let db = DB::new_temp()?;
        let ark = Ark::scan("fixture")?.import_files(&db)?;

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
        let p = db.as_ref().join("cas").join(d.to_hex());

        assert_eq!(d, Digest::from(expected_text));
        assert_eq!(std::fs::read_to_string(p)?, expected_text.to_owned());

        // Get same results if we start from in-mem copy
        assert_eq!(Ark::scan("fixture")?.read()?.import_files(&db)?, ark);

        Ok(())
    }

    #[test]
    fn import() -> Result<()> {
        let db = DB::new_temp()?;
        let digest = Ark::scan("fixture")?.import(&db)?;
        assert_eq!(
            digest.to_hex(),
            "647f1efbfa520cfc16d974d0a1414f5795e58f612bd4928039b7088c347250b8"
        );
        Ok(())
    }

    #[test]
    fn empty_files() -> Result<()> {
        let db = DB::new_temp()?;
        let digest = Ark::scan("src")?.import(&db);
        assert!(digest.is_ok());
        Ok(())
    }
}
