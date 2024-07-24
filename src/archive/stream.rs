use crate::archive::core::*;
use crate::stream::core::Sink;
use std::io::{Result,Read};

pub struct ArchiveSink<'a, S> where S: Storage {
    store: &'a S,
    format: Format,
    compression: Compression,
}
impl<'a, S> ArchiveSink<'a, S> where S: Storage {
    pub fn new(store: &'a S) -> Self {
        Self {
            store: store,
            format: Format::JSON,
            compression: Compression::Plain,
        }
    }
}
impl<S> Sink for ArchiveSink<'_, S> where S: Storage {
    type Receipt = Triad;

    fn send_dir(self, path: impl AsRef<Path>, attrs: Attrs) -> Result<Self> {
        Ok(self)
    }

    fn send_file(self, path: impl AsRef<Path>, attrs: Attrs, r: impl Read) -> Result<Self> {
        Ok(self)
    }

    fn finalize(self) -> Result<Triad> {
        Ok(Triad(self.format, self.compression, Digest::from("foo")))
    }
}

fn archive_source<S>(store: &impl Storage, triad: Triad, sink: S) -> Result<S::Receipt> where S: Sink {
    sink.finalize()
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;
    use indoc::indoc;

    #[test]
    fn round_trip() -> Result<()> {
        use crate::stream::debug::{source,sink};
        use crate::storage::simple::storage;

        let dir = tempdir()?;
        let store = storage(&dir)?;
        let arc_sink = ArchiveSink::new(&store);
        let triad = source(arc_sink)?;

        let mut s = String::new();
        let debug_sink = sink(&mut s);
        archive_source(&store, triad, debug_sink)?;
        assert_eq!(&s, indoc! {""});

        Ok(())
    }
}
