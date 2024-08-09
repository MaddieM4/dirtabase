use crate::archive::core::{Archive, ArchiveFormat, Compression, Triad, TriadFormat};
use crate::storage::traits::*;
use regex::Regex;
use std::io::{Error, Result};
use std::process::Command;
use tempfile::tempdir;

#[derive(Copy, Clone)]
pub struct EncodingSettings(ArchiveFormat, Compression);

pub const DEFAULT_ENCODING: EncodingSettings =
    EncodingSettings(ArchiveFormat::JSON, Compression::Plain);

fn _err<T>(text: &'static str) -> Result<T> {
    Err(Error::other(text))
}

pub struct Context<'a, S>
where
    S: Storage,
{
    store: &'a S,
    encoding: EncodingSettings,
    pub triads: Vec<Triad>,
}

impl<'a, S> Context<'a, S>
where
    S: Storage,
{
    pub fn new(store: &'a S, encoding: EncodingSettings) -> Self {
        Self {
            store: store,
            encoding: encoding,
            triads: vec![],
        }
    }

    pub fn new_from(store: &'a S, triads: Vec<Triad>) -> Self {
        Self {
            store: store,
            encoding: DEFAULT_ENCODING,
            triads: triads,
        }
    }

    fn read(&self, t: &Triad) -> Result<Archive> {
        let (f, c, d) = (t.0, t.1, t.2);
        let f = match f {
            TriadFormat::File => _err("All input triads must be archives"),
            TriadFormat::Archive(f) => Ok(f),
        };
        crate::archive::api::read_archive(f?, c, &d, self.store)
    }

    fn write(&self, ar: &Archive) -> Result<Triad> {
        let (store, f, c) = (self.store, self.encoding.0, self.encoding.1);
        let digest = crate::archive::api::write_archive(ar, f, c, store)?;
        Ok(Triad(TriadFormat::Archive(f), c, digest))
    }

    pub fn finish(mut self) -> Result<Triad> {
        self.triads
            .pop()
            .ok_or(Error::other("Build completed with no triads on stack"))
    }

    // ------------------------------------------------------------------------
    // Operations
    // ------------------------------------------------------------------------

    pub fn empty(mut self) -> Result<Self> {
        self.triads.push(self.write(&vec![])?);
        Ok(self)
    }

    pub fn import(mut self, params: Vec<String>) -> Result<Self> {
        for p in params {
            let sink = crate::stream::archive::sink(self.store);
            let triad = crate::stream::osdir::source(p, sink)?;
            self.triads.push(triad)
        }
        Ok(self)
    }

    pub fn export(mut self, params: Vec<String>) -> Result<Self> {
        if params.len() > self.triads.len() {
            return Err(Error::other(format!(
                "Cannot do {} exports when given only {} input archives",
                params.len(),
                self.triads.len(),
            )));
        }

        let to_export = self.triads.split_off(self.triads.len() - params.len());
        assert_eq!(to_export.len(), params.len());

        for (triad, dir) in std::iter::zip(to_export, params) {
            crate::stream::archive::source(self.store, triad, crate::stream::osdir::sink(dir))?
        }

        Ok(self)
    }

    pub fn merge(mut self) -> Result<Self> {
        let ars = self
            .triads
            .iter()
            .map(|t| self.read(t))
            .collect::<Result<Vec<Archive>>>()?;
        let merged = crate::archive::api::merge(&ars[..]);
        self.triads = vec![self.write(&merged)?];
        Ok(self)
    }

    pub fn filter(mut self, params: Vec<String>) -> Result<Self> {
        if params.len() != 1 {
            return _err("--filter takes exactly 1 param");
        }
        let criteria = Regex::new(&params[0]).map_err(|e| Error::other(e))?;

        let t = self
            .triads
            .pop()
            .ok_or(Error::other("Need an archive to filter"))?;
        let ar = crate::archive::api::filter(self.read(&t)?, &criteria);
        self.triads.push(self.write(&ar)?);
        Ok(self)
    }

    pub fn replace(mut self, params: Vec<String>) -> Result<Self> {
        if params.len() != 2 {
            return _err("--replace takes exactly 2 params (pattern, replacement)");
        }
        let re = Regex::new(&params[0]).map_err(|e| Error::other(e))?;
        let replacement = &params[1];

        let t = self
            .triads
            .pop()
            .ok_or(Error::other("Need an archive to replace on"))?;
        let ar = crate::archive::api::replace(self.read(&t)?, &re, replacement);
        self.triads.push(self.write(&ar)?);
        Ok(self)
    }

    pub fn prefix(self, params: Vec<String>) -> Result<Self> {
        if params.len() != 2 {
            return _err("--prefix takes exactly 2 params (pattern, replacement)");
        }
        fn fix(pre_wanted: &str, input: &str) -> String {
            pre_wanted.to_owned() + input.trim_start_matches("^").trim_start_matches("/")
        }
        let pattern = fix("^/", &params[0]);
        let replacement = fix("/", &params[1]);
        self.replace(vec![pattern, replacement])
    }

    pub fn cmd_impure(mut self, params: Vec<String>) -> Result<Self> {
        if params.len() != 1 {
            return _err("--cmd-impure only takes 1 argument");
        }
        let command = &params[0];

        // Extract to temporary directory
        let t = self
            .triads
            .pop()
            .ok_or(Error::other("Need an archive to work on"))?;
        let dir = tempdir()?;
        crate::stream::archive::source(self.store, t, crate::stream::osdir::sink(&dir))?;

        // Run the command
        // Equivalent to: bash -o pipefail -e -c '...'
        println!("--- [{}] ---", command);
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

        let reimport =
            crate::stream::osdir::source(&dir, crate::stream::archive::sink(self.store))?;
        self.triads.push(reimport);
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::simple::storage;
    use tempfile::tempdir;

    fn build_lua_5_4_7<S>(ctx: Context<S>) -> Result<Triad>
    where
        S: Storage,
    {
        ctx.empty()?
            .cmd_impure(vec!["wget https://www.lua.org/ftp/lua-5.4.7.tar.gz".into()])?
            .cmd_impure(vec!["tar zxf lua-5.4.7.tar.gz".into()])?
            .filter(vec!["^/lua-5.4.7".into()])?
            .prefix(vec!["lua-5.4.7".into(), "".into()])?
            .cmd_impure(vec!["make all test".into()])?
            .finish()
    }

    #[test]
    fn lua_recipe_example() -> Result<()> {
        let store_dir = tempdir()?;
        let store = storage(store_dir.path())?;
        let ctx = Context::new(&store, DEFAULT_ENCODING);
        let triad = build_lua_5_4_7(ctx)?;
        Ok(())
    }
}
