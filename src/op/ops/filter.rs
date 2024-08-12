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
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        let re = regex::Regex::new(&self.0).map_err(|e| Error::other(e))?;
        let t = stack
            .pop()
            .ok_or(Error::other("Need an archive to filter"))?;
        let ar = crate::archive::api::filter(cfg.read_archive(&t)?, &re);
        stack.push(cfg.write_archive(&ar)?);
        Ok(stack)
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
        let store = crate::storage::new_from_tempdir()?;
        let cfg = Config::new(&store);
        let op = Filter("hello".into());

        // Zero input triads
        assert!(op.transform(&cfg, vec![]).is_err());

        // Always filters the top archive on the stack, ignoring lower ones
        let dt = crate::stream::debug::source(crate::stream::archive::sink(&store))?;
        let [rt1, rt2] = random_triads();
        let stack = op.transform(&cfg, vec![rt1, rt2, dt])?;
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
}
