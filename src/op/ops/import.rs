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
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        for path in &self.0 {
            let sink = crate::stream::archive::sink(cfg.store);
            stack.push(crate::stream::osdir::source(path, sink)?);
        }
        Ok(stack)
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
        let cfg = Config::new(&store, &mut log);
        let [rt1, rt2] = random_triads();
        let f = fixture_triad();

        // Zero arguments
        let op = Import::from_args([] as [&str; 0])?;
        assert_eq!(op.transform(&cfg, vec![])?, vec![]);
        assert_eq!(op.transform(&cfg, vec![rt1, rt2])?, vec![rt1, rt2]);

        // One argument
        let op = Import::from_args(["fixture"])?;
        assert_eq!(op.transform(&cfg, vec![])?, vec![f]);
        assert_eq!(op.transform(&cfg, vec![rt1, rt2])?, vec![rt1, rt2, f]);

        // Two arguments
        let op = Import::from_args(["fixture", "fixture"])?;
        assert_eq!(op.transform(&cfg, vec![])?, vec![f, f]);
        assert_eq!(op.transform(&cfg, vec![rt1, rt2])?, vec![rt1, rt2, f, f]);

        Ok(())
    }
}
