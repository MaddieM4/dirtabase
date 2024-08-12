use super::prelude::*;
use std::process::Command;

#[derive(Debug, PartialEq, Clone)]
pub struct CmdImpure(String);

impl FromArgs for CmdImpure {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [url] = unpack("cmd-impure", args, ["url"])?;
        Ok(CmdImpure(url))
    }
}

impl Transform for &CmdImpure {
    fn transform<P>(self, ctx: &mut Context<P>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let command = &self.0;

        // Extract to temporary directory
        let t = ctx
            .stack
            .pop()
            .ok_or(Error::other("Need an archive to work on"))?;
        let dir = tempfile::tempdir()?;
        crate::stream::archive::source(ctx.store, t, crate::stream::osdir::sink(&dir))?;

        // Run the command
        // Equivalent to: bash -o pipefail -e -c '...'
        write!(ctx.log.cmd(), "--- [{}] ---\n", command)?;
        let status = Command::new("bash")
            .arg("-o")
            .arg("pipefail")
            .arg("-e")
            .arg("-c")
            .arg(command)
            .current_dir(&dir)
            .status()?;

        if !&status.success() {
            return Err(Error::other(format!(
                "Command {:?} failed with status {:?}",
                command,
                status.code().unwrap()
            )));
        }

        // Re-import directory back into a new stored archive
        let reimport = crate::stream::osdir::source(&dir, crate::stream::archive::sink(ctx.store))?;
        ctx.stack.push(reimport);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert!(CmdImpure::from_args([] as [&str; 0]).is_err());
        assert!(CmdImpure::from_args(["foo", "bar"]).is_err());
        assert_eq!(CmdImpure::from_args(["foo"])?, CmdImpure("foo".into()));
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let op = CmdImpure("touch grass".into());

        // Let's see it work!
        let sink = crate::stream::archive::sink(&store);
        let dt = crate::stream::debug::source(sink)?;
        let stack = subvert(&store, &mut log).with([dt]).apply(&op)?.stack;
        assert_eq!(
            print_archive(&store, stack[0])?,
            indoc! {"
              FILE /some/dir/hello.txt
                Length: 17
              FILE /grass
                Length: 0
              DIR /some/dir
              DIR /some
              DIR /a/directory
              DIR /a
            "}
        );

        // But does it catch failure? Especially in pipelines?
        let op = CmdImpure("echo hello | exit 60".into());
        assert!(subvert(&store, &mut log).with([dt]).apply(&op).is_err());

        Ok(())
    }
}
