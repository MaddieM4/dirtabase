use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Import(pub Vec<String>);

impl FromArgs for Import {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        Ok(Self(
            args.into_iter().map(|t| t.as_ref().to_owned()).collect(),
        ))
    }
}

impl Transform for &Import {
    fn transform(&self, ctx: &mut Context) -> Result<()> {
        for path in &self.0 {
            let sink = crate::stream::archive::sink(ctx.store);
            ctx.stack.push(crate::stream::osdir::source(path, sink)?);
        }
        Ok(())
    }
}

impl Context<'_> {
    pub fn import<T>(self, args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        write!(self.log.opheader(), "--- Import ---\n")?;
        self.apply(&Import::from_args(args)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert_eq!(Import::from_args(Vec::<String>::new())?, Import(vec![]));
        assert_eq!(
            Import::from_args(["foo", "bar"])?,
            Import(vec!["foo".into(), "bar".into(),])
        );
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let [rt1, rt2] = random_triads();
        let f = fixture_triad();

        // Zero arguments
        let op = Import::from_args([] as [&str; 0])?;
        assert_eq!(ctx(&store, &mut log).apply(&op)?.stack, vec![]);
        assert_eq!(
            ctx(&store, &mut log).with([rt1, rt2]).apply(&op)?.stack,
            vec![rt1, rt2]
        );

        // One argument
        let op = Import::from_args(["fixture"])?;
        assert_eq!(ctx(&store, &mut log).apply(&op)?.stack, vec![f]);
        assert_eq!(
            ctx(&store, &mut log).with([rt1, rt2]).apply(&op)?.stack,
            vec![rt1, rt2, f]
        );

        // Two arguments
        let op = Import::from_args(["fixture", "fixture"])?;
        assert_eq!(ctx(&store, &mut log).apply(&op)?.stack, vec![f, f]);
        assert_eq!(
            ctx(&store, &mut log).with([rt1, rt2]).apply(&op)?.stack,
            vec![rt1, rt2, f, f]
        );

        Ok(())
    }

    #[test]
    fn ctx_extension() -> Result<()> {
        let (store, mut log) = basic_kit();
        let triad = ctx(&store, &mut log).import(["fixture"])?.finish()?;
        assert_eq!(triad, fixture_triad());
        Ok(())
    }
}
