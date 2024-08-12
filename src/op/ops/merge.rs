use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Merge;

impl FromArgs for Merge {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [] = unpack("merge", args, [])?;
        return Ok(Merge);
    }
}

impl Transform for &Merge {
    fn transform<P>(self, cfg: &Config<P>, stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        let ars: Vec<Archive> = stack
            .iter()
            .map(|t| cfg.read_archive(t))
            .collect::<Result<Vec<Archive>>>()?;
        let merged: Archive = crate::archive::api::merge(&ars[..]);
        Ok(vec![cfg.write_archive(&merged)?])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert_eq!(Merge::from_args(Vec::<String>::new())?, Merge);
        assert!(Merge::from_args(["foo", "bar"]).is_err());
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let cfg = Config::new(&store, &mut log);
        let op = Merge;
        let f: Triad =
            crate::stream::osdir::source("fixture", crate::stream::archive::sink(&store))?;

        // Zero input triads
        assert_eq!(op.transform(&cfg, vec![])?, vec![empty_triad()]);

        // Smush down multiple identical triads
        assert_eq!(op.transform(&cfg, vec![f, f, f, f, f])?, vec![f]);

        // Fixture plus empties is also still fixture
        assert_eq!(
            op.transform(&cfg, vec![empty_triad(), f, empty_triad()])?,
            vec![f]
        );

        // Random triads can't be found for reading
        assert!(op
            .transform(&cfg, vec![random_triad(), random_triad()])
            .is_err());

        Ok(())
    }
}
