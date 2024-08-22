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
    pub fn stats(&self, stack_size: usize) -> (usize, usize) {
        match self {
            Op::Empty => (0, 1),
            Op::Import { targets, .. } => (0, targets.len()),
            Op::Export(_) => (1, 0),
            Op::Merge => (stack_size, 1),
            Op::Prefix(_) => (1, 1),
            Op::Filter(_) => (1, 1),
            Op::Rename(_, _) => (1, 1),
            Op::Download(_, _) => (0, 1),
            Op::DownloadImpure(_) => (0, 1),
            Op::CmdImpure(_) => (1, 1),
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

    pub fn cache_digests(&self, ctx: &mut Context) -> Option<Vec<Digest>> {
        let key = self.cache_key();
        let path = ctx.db.join("cache").join(key.to_hex());
        if path.exists() {
            let read = std::fs::read(path).expect("failed to read cache entry");
            let s = String::from_utf8(read).expect("failed to interpret utf-8");
            let d: Vec<Digest> = serde_json::from_str(&s).expect("failed to parse json");
            Some(d)
        } else {
            None
        }
    }
    pub fn can_cache(&self) -> bool {
        match self.0 {
            Op::Download(_, _) => true,
            _ => false,
        }
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

        let cache_digests = self.cache_digests(ctx);
        let has_cache = cache_digests.is_some();
        let can_cache = self.can_cache();
        write!(
            ctx.log.opheader(),
            " + Can cache? {}\n + Is in cache? {}\n",
            can_cache,
            has_cache,
        )?;

        if has_cache {
            ctx.stack.extend(cache_digests.unwrap());
        } else {
            exec_step(ctx, &self.0, &self.1)?;
        }

        if can_cache {
            let n_produced = self.2;
            let pos = &ctx.stack.len() - n_produced;
            let produced_digests = &ctx.stack[pos..];
            let cache_path = ctx.db.join("cache").join(self.cache_key().to_hex());
            std::fs::write(cache_path, serde_json::to_string(produced_digests)?)?;
        }

        for digest in &ctx.stack {
            write!(ctx.log.stack(), "{}\n", digest.to_hex())?;
        }
        Ok(())
    }
}
