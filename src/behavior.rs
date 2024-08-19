use crate::context::Context;
use crate::op::Op;
use arkive::*;
use std::io::{Error, Result};
use std::path::Path;

// Todo: move into prefix op
fn prefix_ark<C>(ark: Ark<C>, prefix: &str) -> Ark<C> {
    let (p, a, c) = ark.decompose();
    let p: Vec<IPR> = p
        .iter()
        .map(|ipr| (prefix.to_owned() + "/" + ipr.as_ref()).to_ipr())
        .collect();
    Ark::compose(std::rc::Rc::new(p), a, c)
}

/// Download a file and save it to the store.
fn download(db: &DB, url: &str) -> Result<Digest> {
    // TODO: db.tempdir()
    let dir = tempfile::tempdir_in(db.join("tmp"))?;
    let mut resp = reqwest::blocking::get(url).map_err(|e| Error::other(e))?;
    let name = url_filename(url)?;
    let dest = dir.path().join(name);
    resp.copy_to(&mut std::fs::File::create(dest)?)
        .map_err(|e| Error::other(e))?;
    Ark::scan(dir.path())?.import(db)
}

/// Derive a filename from parsing a URL.
pub fn url_filename(given_url: &str) -> Result<String> {
    let parsed_url = reqwest::Url::parse(&given_url).map_err(|e| Error::other(e))?;
    Ok(parsed_url
        .path_segments()
        .ok_or(Error::other("Could not break URL into path segments"))?
        .last()
        .ok_or(Error::other("Could not determine filename"))?
        .to_owned())
}

pub fn exec_step(ctx: &mut Context, op: &Op, consumed: &Vec<Digest>) -> Result<()> {
    Ok(match op {
        Op::Empty => {
            ctx.push(Ark::<&str>::empty().save(ctx.db)?);
        }
        Op::Import { base, targets } => {
            for target in targets {
                let real_dir = Path::new(&base).join(target);
                let ark = prefix_ark(Ark::scan(real_dir)?, target);
                ctx.push(ark.import(ctx.db)?);
            }
        }
        Op::Export(base) => {
            assert_eq!(consumed.len(), 1, "Export consumes 1 archive off the stack");
            let digest = consumed[0];
            let ark: Ark<Digest> = Ark::load(ctx.db, &digest)?;
            ark.write(ctx.db, Path::new(base))?;
        }
        Op::Merge => {
            let arks: Result<Vec<Ark<Digest>>> = consumed
                .iter()
                .map(|digest| Ark::load(ctx.db, digest))
                .collect();

            let ark = Ark::from_entries(
                arks?
                    .into_iter()
                    .flat_map(|ark| ark.to_entries())
                    .collect::<Vec<(IPR, Attrs, Contents<Digest>)>>(),
            );
            ctx.push(ark.save(ctx.db)?);
        }
        Op::Download(url, digest_expected) => {
            let digest = download(ctx.db, &url)?;
            if digest != *digest_expected {
                return Err(Error::other("Hash check failed"));
            }
            ctx.push(digest);
        }
        Op::DownloadImpure(url) => {
            ctx.push(download(ctx.db, &url)?);
        }
    })
}

// The flow API for contexts is tested in doc.rs.
impl Context<'_> {
    pub fn empty(&mut self) -> Result<&mut Self> {
        self.apply(&Op::Empty)?;
        Ok(self)
    }

    pub fn import<T, S>(&mut self, base: impl AsRef<str>, targets: T) -> Result<&mut Self>
    where
        T: Into<Vec<S>>,
        S: AsRef<str>,
    {
        self.apply(&Op::Import {
            base: base.as_ref().into(),
            targets: targets
                .into()
                .iter()
                .map(|s| s.as_ref().to_owned())
                .collect(),
        })?;
        Ok(self)
    }

    pub fn export(&mut self, dest: impl AsRef<str>) -> Result<&mut Self> {
        self.apply(&Op::Export(dest.as_ref().to_owned()))?;
        Ok(self)
    }

    pub fn merge(&mut self) -> Result<&mut Self> {
        self.apply(&Op::Merge)?;
        Ok(self)
    }

    pub fn download(&mut self, url: impl AsRef<str>, hash: impl AsRef<str>) -> Result<&mut Self> {
        let digest = Digest::from_hex(hash.as_ref())
            .map_err(|e| crate::op::ParseError::InvalidDigest(hash.as_ref().to_owned(), e))?;
        self.apply(&Op::Download(url.as_ref().to_owned(), digest))?;
        Ok(self)
    }

    pub fn download_impure(&mut self, url: impl AsRef<str>) -> Result<&mut Self> {
        self.apply(&Op::DownloadImpure(url.as_ref().to_owned()))?;
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger::Logger;
    use crate::test_tools::fixture_digest;

    #[test]
    fn empty() -> std::io::Result<()> {
        let db = DB::new_temp()?;
        let mut log = Logger::new_vec();
        let mut ctx = Context::new(&db, &mut log);
        exec_step(&mut ctx, &Op::Empty, &vec![])?;
        assert_eq!(ctx.stack, vec![Ark::<&str>::empty().to_json()?.to_digest()]);
        Ok(())
    }

    #[test]
    fn import() -> std::io::Result<()> {
        let db = DB::new_temp()?;
        let mut log = Logger::new_vec();
        let mut ctx = Context::new(&db, &mut log);
        exec_step(
            &mut ctx,
            &Op::Import {
                base: ".".into(),
                targets: vec!["fixture".into()],
            },
            &vec![],
        )?;
        assert_eq!(ctx.stack, vec![fixture_digest()]);
        Ok(())
    }
}
