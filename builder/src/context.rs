use crate::logger::Logger;
use crate::op::Op;
use ::ark::*;
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

pub struct Context<'a> {
    db: &'a DB,
    log: &'a mut Logger,
    stack: Vec<Digest>,
}

impl<'a> Context<'a> {
    pub fn new(db: &'a DB, log: &'a mut Logger) -> Self {
        Self {
            db: db,
            log: log,
            stack: vec![],
        }
    }

    pub fn apply(&mut self, op: &Op) -> io::Result<()> {
        ReadyStep::from(op, &mut self.stack)?.apply(self)
    }

    pub fn parse_apply(&mut self, args: Vec<String>) -> io::Result<()> {
        let pipeline = crate::op::parse_pipeline(args)?;
        for op in pipeline {
            self.apply(&op)?
        }
        Ok(())
    }
}

impl Op {
    /// How many stack items will this step consume and produce?
    pub fn stats(&self, _stack_size: usize) -> (usize, usize) {
        match self {
            Op::Empty => (0, 1),
            Op::Import { targets, .. } => (0, targets.len()),
        }
    }
}

/// Just the digests that are applicable to this operation.
#[derive(Serialize)]
pub struct ReadyStep(Op, Vec<Digest>, usize);

// TODO: Rendering and conversion
pub struct StackEmpty;
impl From<StackEmpty> for std::io::Error {
    fn from(_: StackEmpty) -> Self {
        Self::other("Tried to pop entries off an empty stack")
    }
}

impl ReadyStep {
    pub fn from(op: &Op, stack: &mut Vec<Digest>) -> Result<Self, StackEmpty> {
        let (consumes, produces) = op.stats(stack.len());
        if stack.len() < consumes {
            Err(StackEmpty)
        } else {
            let pos = stack.len() - consumes;
            Ok(Self(op.clone(), stack.split_off(pos), produces))
        }
    }

    pub fn can_cache(&self) -> bool {
        false
    }
    pub fn cache_key(&self) -> Digest {
        serde_json::to_string(self)
            .expect("Failed to serialize Op")
            .into()
    }

    pub fn apply(&self, ctx: &mut Context) -> io::Result<()> {
        let sep = "================================================================";
        write!(
            ctx.log.opheader(),
            "{}\n{:?}\n{}\n",
            sep,
            self.0.to_code(),
            sep
        )?;

        // TODO HERE: caching
        let produced = self.apply_op(ctx)?;
        ctx.stack.extend(produced);

        for digest in &ctx.stack {
            write!(ctx.log.stack(), "{}\n", digest.to_hex())?;
        }
        Ok(())
    }

    pub fn apply_op(&self, ctx: &mut Context) -> io::Result<Vec<Digest>> {
        let _consumed = &self.1;
        let produced: Vec<Digest> = match &self.0 {
            Op::Empty => {
                vec![Ark::<&str>::empty().save(ctx.db)?]
            }
            Op::Import { base, targets } => targets
                .iter()
                .map(|path| {
                    let real_dir = Path::new(&base).join(path);
                    Ark::scan(real_dir)?.import(ctx.db)
                })
                .collect::<io::Result<Vec<Digest>>>()?,
        };
        Ok(produced)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger::vec_logger;

    fn fixture_digest() -> Digest {
        let db = DB::new_temp().expect("Temp DB");
        let fixture_ark = Ark::scan("../fixture").expect("Scan fixture dir");
        assert_eq!(fixture_ark.len(), 4);
        let digest = fixture_ark.import(&db).expect("Imported to temp DB");
        digest
    }

    #[test]
    fn empty() -> std::io::Result<()> {
        let db = DB::new_temp()?;
        let mut log = vec_logger();
        let mut ctx = Context::new(&db, &mut log);
        ctx.apply(&Op::Empty)?;
        assert_eq!(ctx.stack, vec![Ark::<&str>::empty().to_json()?.to_digest()]);
        Ok(())
    }

    #[test]
    fn import() -> std::io::Result<()> {
        let db = DB::new_temp()?;
        let mut log = vec_logger();
        let mut ctx = Context::new(&db, &mut log);
        ctx.apply(&Op::Import {
            base: "..".into(),
            targets: vec!["fixture".into()],
        })?;
        assert_eq!(ctx.stack, vec![fixture_digest()]);
        Ok(())
    }
}
