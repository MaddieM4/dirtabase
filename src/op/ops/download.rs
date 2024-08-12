use super::prelude::*;
use crate::archive::core::{Compression, Entry};
use crate::attr::Attrs;
use crate::digest::Digest;

#[derive(Debug, PartialEq, Clone)]
pub struct Download(String, Digest);

impl FromArgs for Download {
    fn from_args<T>(args: impl IntoIterator<Item = T>) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let [url, digest] = unpack("download", args, ["url", "digest"])?;
        let digest = Digest::from_hex(digest).map_err(|e| Error::other(e))?;
        Ok(Download(url, digest))
    }
}

impl Transform for &Download {
    fn transform<P>(self, cfg: &Config<P>, mut stack: Stack) -> Result<Stack>
    where
        P: AsRef<Path>,
    {
        let (given_url, expected_digest) = (&self.0, self.1);
        let filename = url_filename(given_url)?;
        let digest = download(cfg.store, given_url)?;
        if digest != expected_digest {
            return Err(Error::other(format!(
                "Expected digest {:?}, got {:?}",
                expected_digest, digest
            )));
        }

        stack.push(cfg.write_archive(&vec![Entry::File {
            path: ("/".to_owned() + &filename).into(),
            attrs: Attrs::new(),
            compression: Compression::Plain,
            digest: digest.clone(),
        }])?);
        Ok(stack)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::test_helpers::*;

    #[test]
    fn from_args() -> Result<()> {
        assert!(Download::from_args([] as [&str; 0]).is_err());
        assert!(Download::from_args(["foo"]).is_err());
        assert!(Download::from_args(["foo", "bar"]).is_err());

        let d = Digest::from("blah blah blah");
        assert_eq!(
            Download::from_args(["foo", &d.to_hex()])?,
            Download("foo".to_owned(), d)
        );
        Ok(())
    }

    #[test]
    fn transform() -> Result<()> {
        let store = crate::storage::new_from_tempdir()?;
        let cfg = Config::new(&store);
        let op = Download(
            "https://gist.githubusercontent.com/MaddieM4/92f0719922db5fbd60a12d762deca9ae/raw/37a4fe4d300b6a88913a808095fd52c1c356030a/reproducible.txt".into(),
            Digest::from("This exists for testing the pure downloads feature of Dirtabase."),
        );

        // Always creates an archive on the top of the stack.
        let [rt1, rt2] = random_triads();
        let stack = op.transform(&cfg, vec![rt1, rt2])?;
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

        // What if we expect the wrong hash?
        let op = Download(op.0, Digest::from("Some other thing"));
        assert!(op.transform(&cfg, vec![]).is_err());

        Ok(())
    }
}
