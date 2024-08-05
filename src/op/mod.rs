use crate::archive::core::Triad;
use crate::storage::traits::Storage;
use std::io::Result;

// TODO: Multi-backend interaction

pub trait Operation {
    fn perform(store: &impl Storage, triads: Vec<Triad>, params: Vec<String>) -> Result<Vec<Triad>>;
}

pub struct Import;
impl Operation for Import {
    fn perform(store: &impl Storage, triads: Vec<Triad>, params: Vec<String>) -> Result<Vec<Triad>> {
        let mut output = triads;
        for p in params {
            let sink = crate::archive::stream::ArchiveSink::new(store);
            let triad = crate::stream::osdir::source(p, sink)?;
            output.push(triad)
        }
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;
    use crate::archive::core::{TriadFormat, Compression, Attrs};
    use crate::storage::simple::storage;
    use crate::stream::core::Sink;
    use crate::digest::Digest;

    fn fixture_triad() -> Result<Triad> {
        let dir = tempdir()?;
        let store = storage(dir.path())?;
        let sink = crate::archive::stream::ArchiveSink::new(&store);

        sink.send_file("/file_at_root.txt", Attrs::new(), std::fs::File::open("fixture/file_at_root.txt")?)?
            .send_dir("/dir1", Attrs::new())?
            .send_dir("/dir1/dir2", Attrs::new())?
            .send_file("/dir1/dir2/nested.txt", Attrs::new(), std::fs::File::open("fixture/dir1/dir2/nested.txt")?)?
            .finalize()
    }

    #[test]
    fn import() -> Result<()> {
        let dir = tempdir()?;
        let store = storage(dir.path())?;
        let t1 = Triad(TriadFormat::File, Compression::Plain, Digest::from("foo"));
        let t2 = Triad(TriadFormat::File, Compression::Plain, Digest::from("bar"));
        let t3 = fixture_triad()?;

        assert_eq!(Import::perform(&store, vec![], vec![])?, vec![]);
        assert_eq!(Import::perform(&store, vec![t1, t2], vec![])?, vec![t1,t2]);
        assert_eq!(Import::perform(&store, vec![t1, t2], vec!["./fixture".into()])?, vec![t1,t2, t3]);
        Ok(())
    }
}
