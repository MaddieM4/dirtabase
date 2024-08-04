use crate::archive::core::*;
use crate::archive::api::*;
use crate::storage::traits::*;
use crate::stream::core::Sink;
use std::io::{Cursor, Error, ErrorKind, Read, Result};

pub struct ArchiveSink<'a, S>
where
    S: Storage,
{
    store: &'a S,
    archive: Archive,
    format: Format,
    compression: Compression,
}
impl<'a, S> ArchiveSink<'a, S>
where
    S: Storage,
{
    pub fn new(store: &'a S) -> Self {
        Self {
            store: store,
            archive: vec![],
            format: Format::JSON,
            compression: Compression::Plain,
        }
    }
}
impl<S> Sink for ArchiveSink<'_, S>
where
    S: Storage,
{
    type Receipt = Triad;

    fn send_dir(mut self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self> {
        // This is an awful dumb hack that I'm just using to get to
        // a clean refactor milestone. It's about to not work this way.
        //
        // What's going on here? Storing an empty archive serialized as JSON.
        // Which we can then reference as a sub-archive to work around the
        // temporary limitation of archives ONLY being able to represent a dir
        // as a sub-archive. The cleaner future solution is going to work by
        // making the archive format match streams exactly, then extend with
        // support for sub-archives.

        let digest = self.store.cas().write(Cursor::new("[]"))?;
        let triad = Triad(Format::JSON, Compression::Plain, digest);
        let entry = Entry(path.as_ref().into(), triad, attrs);
        self.archive.push(entry);
        Ok(self)
    }

    fn send_file(mut self, path: impl AsRef<Path>, attrs: Attrs, r: impl Read) -> Result<Self> {
        let digest = self.store.cas().write(r)?;
        let triad = Triad(Format::File, Compression::Plain, digest);
        let entry = Entry(path.as_ref().into(), triad, attrs);
        self.archive.push(entry);
        Ok(self)
    }

    fn finalize(self) -> Result<Triad> {
        let bytes = archive_encode(&self.archive, self.format, self.compression)?;
        let digest = self.store.cas().write(Cursor::new(bytes))?;
        Ok(Triad(self.format, self.compression, digest))
    }
}

pub fn archive_source<S>(store: &impl Storage, triad: Triad, mut sink: S) -> Result<S::Receipt>
where
    S: Sink,
{
    let opt_reader = store.cas().read(triad.digest())?;
    let mut r = opt_reader.ok_or(Error::new(
        ErrorKind::NotFound,
        "Source digest doesn't exist in store",
    ))?;

    let mut bytes: Vec<u8> = vec![];
    r.read_to_end(&mut bytes)?;

    let archive = archive_decode(bytes, triad.format(), triad.compression())?;
    for entry in archive {
        let triad: Triad = entry.1;
        if triad.format() == Format::File {
            let opt_reader = store.cas().read(entry.1.digest())?;
            let r = opt_reader.ok_or(Error::new(
                ErrorKind::NotFound,
                "Source digest doesn't exist in store",
            ))?;
            sink = sink.send_file(entry.0, entry.2, r)?;
        } else {
            // TODO: recursion
            sink = sink.send_dir(entry.0, entry.2)?;
        }
    }

    sink.finalize()
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip() -> Result<()> {
        use crate::storage::simple::storage;
        use crate::stream::debug;

        let dir = tempdir()?;
        let store = storage(&dir)?;
        let arc_sink = ArchiveSink::new(&store);
        let triad = debug::source(arc_sink)?;

        let txt = archive_source(&store, triad, debug::sink())?;
        assert_eq!(txt, debug::source(debug::sink())?);

        Ok(())
    }
}
