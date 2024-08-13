use super::prelude::*;
use crate::archive::core::{Compression, Entry};
use crate::attr::Attrs;

#[derive(Debug, PartialEq, Clone)]
pub struct DownloadImpure(String);

impl FromArgs for DownloadImpure {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [url] = unpack("download-impure", args, ["url"])?;
        Ok(DownloadImpure(url))
    }
}

impl Transform for &DownloadImpure {
    fn transform(&self, ctx: &mut Context) -> Result<()> {
        let given_url = &self.0;
        let filename = url_filename(given_url)?;
        let digest = download(ctx.store, given_url)?;
        ctx.stack.push(ctx.write_archive(&vec![Entry::File {
            path: ("/".to_owned() + &filename).into(),
            attrs: Attrs::new(),
            compression: Compression::Plain,
            digest: digest.clone(),
        }])?);
        Ok(())
    }
}

impl Context<'_> {
    pub fn download_impure(self, url: &str) -> Result<Self> {
        write!(self.log.opheader(), "--- DownloadImpure ---\n")?;
        self.apply(&DownloadImpure(url.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert!(DownloadImpure::from_args([] as [&str; 0]).is_err());
        assert!(DownloadImpure::from_args(["foo", "bar"]).is_err());
        assert_eq!(
            DownloadImpure::from_args(["foo"])?,
            DownloadImpure("foo".into())
        );
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let (store, mut log) = basic_kit();
        let op = DownloadImpure(
            "https://gist.githubusercontent.com/MaddieM4/92f0719922db5fbd60a12d762deca9ae/raw/37a4fe4d300b6a88913a808095fd52c1c356030a/reproducible.txt".into(),
        );

        // Always creates an archive on the top of the stack.
        let [rt1, rt2] = random_triads();
        let stack = ctx(&store, &mut log).with([rt1, rt2]).apply(&op)?.stack;
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0], rt1);
        assert_eq!(stack[1], rt2);
        assert_eq!(
            print_archive(&store, stack[2])?,
            indoc! {"
          FILE /reproducible.txt
            Length: 64
        "}
        );

        Ok(())
    }

    #[test]
    fn ctx_extension() -> Result<()> {
        let (store, mut log) = basic_kit();
        let triad = ctx(&store, &mut log).download_impure(
            "https://gist.githubusercontent.com/MaddieM4/92f0719922db5fbd60a12d762deca9ae/raw/37a4fe4d300b6a88913a808095fd52c1c356030a/reproducible.txt",
        )?.finish()?;
        assert_eq!(
            print_archive(&store, triad)?,
            indoc! {"
          FILE /reproducible.txt
            Length: 64
        "}
        );
        Ok(())
    }
}
