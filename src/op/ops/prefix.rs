use super::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Prefix(String, String);

fn fix(pre_wanted: &str, input: &str) -> String {
    pre_wanted.to_owned() + input.trim_start_matches("^").trim_start_matches("/")
}

impl FromArgs for Prefix {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [pattern, replacement] = unpack("prefix", args, ["pattern", "replacement"])?;
        return Ok(Prefix(pattern, replacement));
    }
}

impl Transform for &Prefix {
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        let pattern = fix("^/", &self.0);
        let replacement = fix("/", &self.1);

        let re = regex::Regex::new(&pattern).map_err(|e| Error::other(e))?;
        let t = stack
            .pop()
            .ok_or(Error::other("Need an archive to prefix on"))?;
        let ar = crate::archive::api::replace(cfg.read_archive(&t)?, &re, &replacement);
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
        assert!(Prefix::from_args([] as [&str; 0]).is_err());
        assert!(Prefix::from_args(["foo"]).is_err());
        assert_eq!(
            Prefix::from_args(["foo", "bar"])?,
            Prefix("foo".to_owned(), "bar".to_owned())
        );
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let cfg = Config::new(&store, &mut log);
        let op = Prefix("some".into(), "deep/old".into());
        let dt = crate::stream::debug::source(crate::stream::archive::sink(&store))?;
        let [rt1, rt2] = random_triads();

        // Zero input triads
        assert!(op.transform(&cfg, vec![]).is_err());

        // Always prefixs the top archive on the stack, ignoring lower ones
        let stack = op.transform(&cfg, vec![rt1, rt2, dt])?;
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0], rt1);
        assert_eq!(stack[1], rt2);
        assert_eq!(
            print_archive(&store, stack[2])?,
            indoc! {"
          FILE /deep/old/dir/hello.txt
            Length: 17
            AnotherAttr: for example purposes
          DIR /a/directory
            Foo: Bar
        "}
        );

        // Don't replace stuff deeper into the path
        let op = Prefix("hello".into(), "goodbye".into());
        let stack = op.transform(&cfg, vec![dt])?;
        assert_eq!(
            print_archive(&store, stack[0])?,
            indoc! {"
          FILE /some/dir/hello.txt
            Length: 17
            AnotherAttr: for example purposes
          DIR /a/directory
            Foo: Bar
        "}
        );

        // It should be valid if someone includes some ^ and / in the strings
        let op = Prefix("/a".into(), "^/another".into());
        let stack = op.transform(&cfg, vec![dt])?;
        assert_eq!(
            print_archive(&store, stack[0])?,
            indoc! {"
          FILE /some/dir/hello.txt
            Length: 17
            AnotherAttr: for example purposes
          DIR /another/directory
            Foo: Bar
        "}
        );

        Ok(())
    }
}
