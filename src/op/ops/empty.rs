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
    fn transform(&self, ctx: &mut Context) -> Result<()> {
        ctx.stack.push(ctx.write_archive(&vec![])?);
        Ok(())
    }
}

impl Context<'_> {
    pub fn empty(self) -> Result<Self> {
        write!(self.log.opheader(), "--- Empty ---\n")?;
        self.apply(&Empty)
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
        let (store, mut log) = basic_kit();
        let op = Empty;
        let [rt1, rt2, rt3] = random_triads();

        // Zero input triads
        assert_eq!(ctx(&store, &mut log).apply(&op)?.stack, vec![empty_triad()]);

        // Always appends
        assert_eq!(
            ctx(&store, &mut log)
                .with([rt1, rt2, rt3])
                .apply(&op)?
                .stack,
            vec![rt1, rt2, rt3, empty_triad()]
        );

        Ok(())
    }

    #[test]
    fn ctx_extension() -> Result<()> {
        let (store, mut log) = basic_kit();
        assert_eq!(ctx(&store, &mut log).empty()?.stack, vec![empty_triad()]);
        Ok(())
    }
}
