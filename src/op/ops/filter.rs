use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Filter(String);

impl FromArgs for Filter {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [pattern] = unpack("filter", args, ["pattern"])?;
        return Ok(Filter(pattern));
    }
}

impl Transform for &Filter {
    fn transform<P>(&self, ctx: &mut Context<P>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let re = regex::Regex::new(&self.0).map_err(|e| Error::other(e))?;
        let t = ctx
            .stack
            .pop()
            .ok_or(Error::other("Need an archive to filter"))?;
        let ar = crate::archive::api::filter(ctx.read_archive(&t)?, &re);
        ctx.stack.push(ctx.write_archive(&ar)?);
        Ok(())
    }
}

impl<P> crate::op::helpers::Context<'_, P>
where
    P: AsRef<Path>,
{
    pub fn filter(self, pattern: &str) -> Result<Self> {
        write!(self.log.opheader(), "--- Filter ---\n")?;
        self.apply(&Filter(pattern.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert!(Filter::from_args([] as [&str; 0]).is_err());
        assert!(Filter::from_args(["foo", "bar"]).is_err());
        assert_eq!(Filter::from_args(["foo"])?, Filter("foo".to_owned()));
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let op = Filter("hello".into());

        // Zero input triads
        assert!(ctx(&store, &mut log).apply(&op).is_err());

        // Always filters the top archive on the stack, ignoring lower ones
        let dt = crate::stream::debug::source(crate::stream::archive::sink(&store))?;
        let [rt1, rt2] = random_triads();
        let stack = ctx(&store, &mut log).with([rt1, rt2, dt]).apply(&op)?.stack;
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0], rt1);
        assert_eq!(stack[1], rt2);

        let sink = crate::stream::debug::sink();
        let txt = crate::stream::archive::source(&store, stack[2], sink)?;
        assert_eq!(
            txt,
            indoc! {"
                FILE /some/dir/hello.txt
                  Length: 17
                  AnotherAttr: for example purposes
            "}
        );

        Ok(())
    }

    #[test]
    fn ctx_extension() -> Result<()> {
        let (store, mut log) = basic_kit();
        let sink = crate::stream::archive::sink(&store);
        let dt = crate::stream::debug::source(sink)?;
        let triad = ctx(&store, &mut log).with([dt]).filter("hello")?.finish()?;
        assert_eq!(
            print_archive(&store, triad)?,
            indoc! {"
              FILE /some/dir/hello.txt
                Length: 17
                AnotherAttr: for example purposes
            "}
        );
        Ok(())
    }
}
