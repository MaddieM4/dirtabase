use crate::behavior::exec_step;
use crate::logger::Logger;
use crate::op::Op;
use arkive::*;
use serde::Serialize;
use std::io::{self, Write};

pub struct Context<'a> {
    pub db: &'a DB,
    pub log: &'a mut Logger,
    pub stack: Vec<Digest>,
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

    pub fn push(&mut self, digest: Digest) {
        self.stack.push(digest)
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
        exec_step(ctx, &self.0, &self.1)?;

        for digest in &ctx.stack {
            write!(ctx.log.stack(), "{}\n", digest.to_hex())?;
        }
        Ok(())
    }
}
