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
    fn transform(&self, ctx: &mut Context) -> Result<()> {
        let ars: Vec<Archive> = ctx
            .stack
            .iter()
            .map(|t| ctx.read_archive(t))
            .collect::<Result<Vec<Archive>>>()?;
        let merged: Archive = crate::archive::api::merge(&ars[..]);
        ctx.stack = vec![ctx.write_archive(&merged)?];
        Ok(())
    }
}

impl Context<'_> {
    pub fn merge(self) -> Result<Self> {
        write!(self.log.opheader(), "--- Merge ---\n")?;
        self.apply(&Merge)
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
        let op = Merge;
        let e = empty_triad();
        let f: Triad =
            crate::stream::osdir::source("fixture", crate::stream::archive::sink(&store))?;

        // Zero input triads
        assert_eq!(ctx(&store, &mut log).apply(&op)?.stack, vec![empty_triad()]);

        // Smush down multiple identical triads
        assert_eq!(
            ctx(&store, &mut log)
                .with([f, f, f, f, f])
                .apply(&op)?
                .stack,
            vec![f]
        );

        // Fixture plus empties is also still fixture
        assert_eq!(
            ctx(&store, &mut log).with([e, f, e]).apply(&op)?.stack,
            vec![f]
        );

        // Random triads can't be found for reading
        assert!(ctx(&store, &mut log)
            .with([random_triad(), random_triad()])
            .apply(&op)
            .is_err());

        Ok(())
    }

    #[test]
    fn ctx_extension() -> Result<()> {
        let (store, mut log) = basic_kit();
        let f: Triad =
            crate::stream::osdir::source("fixture", crate::stream::archive::sink(&store))?;
        assert_eq!(
            ctx(&store, &mut log).with([f]).empty()?.merge()?.stack,
            vec![f]
        );
        Ok(())
    }
}
