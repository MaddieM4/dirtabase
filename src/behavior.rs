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
    /*
    let name: String = match resp.headers().get("Content-Disposition") {
        Some(header) => todo!(),
        None => url_filename(url)?,
    };
    */

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

    pub fn download_impure(&mut self, url: impl AsRef<str>) -> Result<&mut Self> {
        self.apply(&Op::DownloadImpure(url.as_ref().to_owned()))?;
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger::Logger;

    fn fixture_digest() -> Digest {
        let db = DB::new_temp().expect("Temp DB");
        let fixture_ark = prefix_ark(Ark::scan("fixture").expect("Scan fixture dir"), "fixture");
        assert_eq!(fixture_ark.len(), 4);
        let digest = fixture_ark.import(&db).expect("Imported to temp DB");
        digest
    }

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
