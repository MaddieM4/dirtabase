use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Replace(String, String);

impl FromArgs for Replace {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [pattern, replacement] = unpack("replace", args, ["pattern", "replacement"])?;
        return Ok(Replace(pattern, replacement));
    }
}

impl Transform for &Replace {
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        let re = regex::Regex::new(&self.0).map_err(|e| Error::other(e))?;
        let replacement = &self.1;
        let t = stack
            .pop()
            .ok_or(Error::other("Need an archive to replace on"))?;
        let ar = crate::archive::api::replace(cfg.read_archive(&t)?, &re, replacement);
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
        assert!(Replace::from_args([] as [&str; 0]).is_err());
        assert!(Replace::from_args(["foo"]).is_err());
        assert_eq!(
            Replace::from_args(["foo", "bar"])?,
            Replace("foo".to_owned(), "bar".to_owned())
        );
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let cfg = Config::new(&store, &mut log);
        let op = Replace("hello".into(), "goodbye".into());

        // Zero input triads
        assert!(op.transform(&cfg, vec![]).is_err());

        // Always replaces the top archive on the stack, ignoring lower ones
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
                FILE /some/dir/goodbye.txt
                  Length: 17
                  AnotherAttr: for example purposes
                DIR /a/directory
                  Foo: Bar
            "}
        );

        Ok(())
    }
}
