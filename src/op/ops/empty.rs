use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Empty;

impl FromArgs for Empty {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [] = unpack("empty", args, [])?;
        return Ok(Empty);
    }
}

impl Transform for &Empty {
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        stack.push(cfg.write_archive(&vec![])?);
        Ok(stack)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert_eq!(Empty::from_args(Vec::<String>::new())?, Empty);
        assert!(Empty::from_args(["foo", "bar"]).is_err());
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let cfg = Config::new(&store);
        let op = Empty;
        let [rt1, rt2, rt3] = random_triads();

        // Zero input triads
        assert_eq!(op.transform(&cfg, vec![])?, vec![empty_triad()]);

        // Always appends
        assert_eq!(
            op.transform(&cfg, vec![rt1, rt2, rt3])?,
            vec![rt1, rt2, rt3, empty_triad()]
        );

        Ok(())
    }
}
