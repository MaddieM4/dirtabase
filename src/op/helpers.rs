use crate::archive::core::{Archive, Triad, TriadFormat};
use crate::enc::Settings as Enc;
use crate::logger::Logger;
use crate::op::gen::Op;
use crate::storage::simple::SimpleStorage;
use std::io::{Error, Result, Write};
use std::path::Path;

pub struct Config<'a, P>
where
    P: AsRef<Path>,
{
    pub store: &'a SimpleStorage<P>,
    pub enc: Enc,
    pub log: &'a mut Logger,
}

impl<'a, P> Config<'a, P>
where
    P: AsRef<Path>,
{
    pub fn new(store: &'a SimpleStorage<P>, log: &'a mut Logger) -> Self {
        Self {
            store: store,
            enc: Enc::default(),
            log: log,
        }
    }

    pub fn ctx(&'a mut self) -> Context<'a, P> {
        self.ctx_with(vec![])
    }

    pub fn ctx_with(&'a mut self, stack: impl Into<Vec<Triad>>) -> Context<'a, P> {
        Context {
            store: self.store,
            enc: self.enc,
            log: self.log,
            stack: stack.into(),
        }
    }

    pub fn build(
        &'a mut self,
        stack: impl Into<Stack>,
        transformer: impl Transform,
    ) -> Result<Stack> {
        Ok(self.ctx_with(stack).apply(transformer)?.stack)
    }
}

pub fn subvert<'a, P>(store: &'a SimpleStorage<P>, log: &'a mut Logger) -> Context<'a, P>
where
    P: AsRef<Path>,
{
    Context {
        store: store,
        enc: Enc::default(),
        log: log,
        stack: vec![],
    }
}

pub type Stack = Vec<Triad>;

pub struct Context<'a, P>
where
    P: AsRef<Path>,
{
    pub store: &'a SimpleStorage<P>,
    pub enc: Enc,
    pub log: &'a mut Logger,
    pub stack: Vec<Triad>,
}

impl<'a, P> Context<'a, P>
where
    P: AsRef<Path>,
{
    pub fn with(mut self, triads: impl IntoIterator<Item = Triad>) -> Self {
        self.stack.extend(triads);
        self
    }

    pub fn apply(mut self, item: impl Transform) -> Result<Self> {
        item.transform(&mut self)?;
        Ok(self)
    }

    pub fn parse_apply<S>(self, params: impl IntoIterator<Item = S>) -> Result<Self>
    where
        Self: Sized,
        S: AsRef<str>,
    {
        self.apply(crate::op::parse::parse(params)?)
    }

    pub fn finish(mut self) -> Result<Triad> {
        self.stack
            .pop()
            .ok_or(Error::other("No archives on the stack"))
    }

    pub fn read_archive(&self, t: &Triad) -> Result<Archive> {
        let (f, c, d) = (t.0, t.1, t.2);
        let f = match f {
            TriadFormat::File => Err(Error::other("All input triads must be archives")),
            TriadFormat::Archive(f) => Ok(f),
        };
        crate::archive::api::read_archive(f?, c, &d, self.store)
    }

    pub fn write_archive(&self, ar: &Archive) -> Result<Triad> {
        let (store, f, c) = (self.store, self.enc.f(), self.enc.c());
        let digest = crate::archive::api::write_archive(ar, f, c, store)?;
        Ok(Triad(TriadFormat::Archive(f), c, digest))
    }
}

pub trait Transform {
    fn transform<P>(&self, ctx: &mut Context<P>) -> Result<()>
    where
        P: AsRef<Path>;

    fn header_name(&self) -> &'static str {
        "Unknown"
    }
}

impl<const N: usize> Transform for [Op; N] {
    fn transform<P>(&self, ctx: &mut Context<P>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for item in self {
            write!(ctx.log.opheader(), "--- {} ---\n", item.header_name())?;
            item.transform(ctx)?;
        }
        Ok(())
    }
}

impl Transform for Vec<Op> {
    fn transform<P>(&self, ctx: &mut Context<P>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for item in self {
            write!(ctx.log.opheader(), "--- {} ---\n", item.header_name())?;
            item.transform(ctx)?;
            for triad in &ctx.stack {
                write!(ctx.log.stack(), "{}\n", triad)?;
            }
        }
        Ok(())
    }
}

pub trait FromArgs {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
        Self: Sized;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn simple_ctx_example() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let mut log = crate::logger::vec_logger();

        let t: Triad = subvert(&store, &mut log)
            .apply(&crate::op::ops::import::Import(vec!["fixture".to_owned()]))?
            .finish()?;
        assert_eq!(t, fixture_triad());
        Ok(())
    }

    #[test]
    fn ctx_apply_op_enum() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let mut log = crate::logger::vec_logger();
        let op = Op::Import(crate::op::ops::import::Import(vec!["fixture".to_owned()]));

        let ctx = subvert(&store, &mut log).apply(op)?;
        assert_eq!(ctx.stack, vec![fixture_triad()]);
        Ok(())
    }

    #[test]
    fn ctx_apply_op_seq() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let mut log = crate::logger::vec_logger();
        let op = Op::Import(crate::op::ops::import::Import(vec!["fixture".to_owned()]));

        let ctx = subvert(&store, &mut log).apply([op.clone(), op])?;
        assert_eq!(ctx.stack, vec![fixture_triad(), fixture_triad()]);
        Ok(())
    }

    #[test]
    fn ctx_apply_op_parsed() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let mut log = crate::logger::vec_logger();
        let ctx = subvert(&store, &mut log).parse_apply(["--import", "fixture", "fixture"])?;
        assert_eq!(ctx.stack, vec![fixture_triad(), fixture_triad()]);

        let ctx = subvert(&store, &mut log).parse_apply(["foo"]);
        assert!(ctx.is_err());

        Ok(())
    }
}
