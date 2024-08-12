use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Export(pub Vec<String>);

impl FromArgs for Export {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        Ok(Self(
            args.into_iter().map(|t| t.as_ref().to_owned()).collect(),
        ))
    }
}

impl Transform for &Export {
    fn transform<P>(&self, ctx: &mut Context<P>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let params = &self.0;
        if params.len() > ctx.stack.len() {
            return Err(Error::other(format!(
                "Cannot do {} exports when given only {} input archives",
                params.len(),
                ctx.stack.len(),
            )));
        }

        let to_export = ctx.stack.split_off(ctx.stack.len() - params.len());
        assert_eq!(to_export.len(), params.len());

        for (triad, dir) in std::iter::zip(to_export, params) {
            crate::stream::archive::source(ctx.store, triad, crate::stream::osdir::sink(dir))?
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert_eq!(Export::from_args(Vec::<String>::new())?, Export(vec![]));
        assert_eq!(
            Export::from_args(["foo", "bar"])?,
            Export(vec!["foo".into(), "bar".into()])
        );
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let out = tempfile::tempdir()?;
        let dt = crate::stream::debug::source(crate::stream::archive::sink(&store))?;
        let [rt1, rt2] = random_triads();

        // Zero arguments
        let op = Export::from_args([] as [&str; 0])?;
        assert_eq!(subvert(&store, &mut log).apply(&op)?.stack, vec![]);
        assert_eq!(
            subvert(&store, &mut log).with([rt1, rt2]).apply(&op)?.stack,
            vec![rt1, rt2]
        );

        // One argument, zero stack - should fail
        let op = Export::from_args([out.path().to_string_lossy()])?;
        assert!(subvert(&store, &mut log).apply(&op).is_err());

        // One argument, one stack
        assert_eq!(
            subvert(&store, &mut log).with([dt]).apply(&op)?.stack,
            vec![]
        );
        assert!(out.path().join("some/dir/hello.txt").exists());

        Ok(())
    }
}
