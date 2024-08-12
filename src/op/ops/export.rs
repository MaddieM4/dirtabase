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
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        let params = &self.0;
        if params.len() > stack.len() {
            return Err(Error::other(format!(
                "Cannot do {} exports when given only {} input archives",
                params.len(),
                stack.len(),
            )));
        }

        let to_export = stack.split_off(stack.len() - params.len());
        assert_eq!(to_export.len(), params.len());

        for (triad, dir) in std::iter::zip(to_export, params) {
            crate::stream::archive::source(cfg.store, triad, crate::stream::osdir::sink(dir))?
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
        let cfg = Config::new(&store, &mut log);
        let out = tempfile::tempdir()?;
        let dt = crate::stream::debug::source(crate::stream::archive::sink(&store))?;
        let [rt1, rt2] = random_triads();

        // Zero arguments
        let op = Export::from_args([] as [&str; 0])?;
        assert_eq!(op.transform(&cfg, vec![])?, vec![]);
        assert_eq!(op.transform(&cfg, vec![rt1, rt2])?, vec![rt1, rt2]);

        // One argument, zero stack - should fail
        let op = Export::from_args([out.path().to_string_lossy()])?;
        assert!(op.transform(&cfg, vec![]).is_err());

        // One argument, one stack
        assert_eq!(op.transform(&cfg, vec![dt])?, vec![]);
        assert!(out.path().join("some/dir/hello.txt").exists());

        Ok(())
    }
}
