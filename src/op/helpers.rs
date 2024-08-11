use crate::archive::core::Triad;
use crate::enc::Settings as Enc;
use crate::storage::simple::SimpleStorage;
use std::io::{Error, Result};
use std::path::Path;

pub struct Config<'a, P>
where
    P: AsRef<Path>,
{
    pub store: &'a SimpleStorage<P>,
    pub enc: Enc,
}

impl<'a, P> Config<'a, P>
where
    P: AsRef<Path>,
{
    pub fn new(store: &'a SimpleStorage<P>) -> Self {
        Self {
            store: store,
            enc: Enc::default(),
        }
    }

    pub fn ctx(&'a self) -> Context<'a, P> {
        Context(self, vec![])
    }
}

pub type Stack = Vec<Triad>;

pub struct Context<'a, P>(&'a Config<'a, P>, Stack)
where
    P: AsRef<Path>;

impl<'a, P> Context<'a, P>
where
    P: AsRef<Path>,
{
    pub fn finish(mut self) -> Result<Triad> {
        self.1.pop().ok_or(Error::other("No archives on the stack"))
    }
}

trait Apply<T> {
    fn apply(self, item: T) -> Result<Self>
    where
        Self: Sized;
}

impl<'a, P, T> Apply<T> for Context<'a, P>
where
    P: AsRef<Path>,
    T: Operation,
{
    fn apply(self, item: T) -> Result<Self> {
        Ok(Self(self.0, item.transform(self.0, self.1)?))
    }
}

pub trait Operation {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
        Self: Sized;

    fn transform<P>(&self, cfg: &Config<P>, stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn simple_ctx_example() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let cfg = Config::new(&store);

        let t: Triad = cfg
            .ctx()
            .apply(crate::op::ops::import::Import(vec!["fixture".to_owned()]))?
            .finish()?;
        assert_eq!(t, fixture_triad());
        Ok(())
    }
}
