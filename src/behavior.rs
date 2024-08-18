use crate::context::Context;
use crate::op::Op;
use arkive::*;
use std::io::Result;
use std::path::Path;

pub fn exec_step(ctx: &mut Context, op: &Op, _consumed: &Vec<Digest>) -> Result<()> {
    Ok(match op {
        Op::Empty => {
            ctx.push(Ark::<&str>::empty().save(ctx.db)?);
        }
        Op::Import { base, targets } => {
            for target in targets {
                let real_dir = Path::new(&base).join(target);
                ctx.push(Ark::scan(real_dir)?.import(ctx.db)?);
            }
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger::Logger;

    fn fixture_digest() -> Digest {
        let db = DB::new_temp().expect("Temp DB");
        let fixture_ark = Ark::scan("fixture").expect("Scan fixture dir");
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
