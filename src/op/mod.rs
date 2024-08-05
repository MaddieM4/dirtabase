use crate::archive::core::{Triad,TriadFormat};
use crate::storage::traits::*;
use std::io::{Error, Result};
use regex::Regex;

// TODO: Multi-backend interaction

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    Import,
    Export,
    Filter,
}

pub fn perform(
    op: Op,
    store: &impl Storage,
    triads: Vec<Triad>,
    params: Vec<String>,
) -> Result<Vec<Triad>> {
    match op {
        Op::Import => import(store, triads, params),
        Op::Export => export(store, triads, params),
        Op::Filter => filter(store, triads, params),
    }
}

fn import(store: &impl Storage, triads: Vec<Triad>, params: Vec<String>) -> Result<Vec<Triad>> {
    let mut output = triads;
    for p in params {
        let sink = crate::archive::stream::ArchiveSink::new(store);
        let triad = crate::stream::osdir::source(p, sink)?;
        output.push(triad)
    }
    Ok(output)
}

fn export(store: &impl Storage, triads: Vec<Triad>, params: Vec<String>) -> Result<Vec<Triad>> {
    if params.len() > triads.len() {
        return Err(Error::other(format!(
            "Cannot do {} exports when given only {} input archives",
            params.len(),
            triads.len(),
        )));
    }

    let mut output = triads;
    let to_export = output.split_off(output.len() - params.len());
    assert_eq!(to_export.len(), params.len());

    for (triad, dir) in std::iter::zip(to_export, params) {
        crate::archive::stream::archive_source(
            store,
            triad,
            crate::stream::osdir::sink(dir))?
    }

    Ok(output)
}

fn filter(store: &impl Storage, triads: Vec<Triad>, params: Vec<String>) -> Result<Vec<Triad>> {
    if params.len() != 1 {
        return Err(Error::other("--filter takes exactly 1 param"));
    }
    let criteria = Regex::new(&params[0]).map_err(|e| Error::other(e))?;
    let mut output: Vec<Triad> = vec![];
    for t in triads {
        let (f, c, d) = (t.0, t.1, t.2);
        let f = match f {
            TriadFormat::File => return Err(Error::other("All input triads must be archives")),
            TriadFormat::Archive(f) => f,
        };
        let ar = crate::archive::api::read_archive(f, c, &d, store)?;
        let ar = crate::archive::api::filter(ar, &criteria);
        let digest = crate::archive::api::write_archive(&ar, f, c, store)?;
        output.push(Triad(TriadFormat::Archive(f), c, digest));
    }
    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::archive::core::{Attrs, Compression};
    use crate::digest::Digest;
    use crate::storage::simple::storage;
    use crate::stream::core::Sink;
    use tempfile::tempdir;

    fn fixture_triad() -> Result<Triad> {
        let dir = tempdir()?;
        let store = storage(dir.path())?;
        let sink = crate::archive::stream::ArchiveSink::new(&store);

        sink.send_file(
            "/file_at_root.txt",
            Attrs::new(),
            std::fs::File::open("fixture/file_at_root.txt")?,
        )?
        .send_dir("/dir1", Attrs::new())?
        .send_dir("/dir1/dir2", Attrs::new())?
        .send_file(
            "/dir1/dir2/nested.txt",
            Attrs::new(),
            std::fs::File::open("fixture/dir1/dir2/nested.txt")?,
        )?
        .finalize()
    }

    // TODO: move to utils
    fn path_str(p: impl AsRef<std::path::Path>) -> String {
        p.as_ref()
            .to_str()
            .expect("Could not convert path to string")
            .into()
    }

    #[test]
    fn import() -> Result<()> {
        let op = Op::Import;
        let dir = tempdir()?;
        let store = storage(dir.path())?;
        let t1 = Triad(TriadFormat::File, Compression::Plain, Digest::from("foo"));
        let t2 = Triad(TriadFormat::File, Compression::Plain, Digest::from("bar"));
        let t3 = fixture_triad()?;

        assert_eq!(perform(op, &store, vec![], vec![])?, vec![]);
        assert_eq!(perform(op, &store, vec![t1, t2], vec![])?, vec![t1, t2]);
        assert_eq!(
            perform(op, &store, vec![t1, t2], vec!["./fixture".into()])?,
            vec![t1, t2, t3]
        );
        Ok(())
    }

    #[test]
    fn export() -> Result<()> {
        let op = Op::Export;
        let dir = tempdir()?;
        let store = storage(dir.path())?;
        let mut imported = perform(Op::Import, &store, vec![], vec!["./fixture".into()])?;
        let t: Triad = imported.pop().unwrap();

        let output_dir = tempdir()?;
        assert_eq!(
            perform(op, &store, vec![t], vec![path_str(&output_dir)])?,
            vec![]
        );
        assert!(output_dir.path().join("dir1/dir2/nested.txt").exists());
        Ok(())
    }

    #[test]
    fn filter() -> Result<()> {
        let out = tempdir()?;
        let store_dir = tempdir()?;
        let store = storage(store_dir.path())?;
        let imported = perform(Op::Import, &store, vec![], vec!["./fixture".into()])?;
        let filtered = perform(Op::Filter, &store, imported, vec!["dir2".into()])?;
        let exported = perform(Op::Export, &store, filtered, vec![path_str(&out)])?;

        assert_eq!(exported, vec![]);
        assert!(out.path().join("dir1/dir2").exists());
        assert!(out.path().join("dir1/dir2/nested.txt").exists());
        assert!(! out.path().join("file_at_root.txt").exists());
        Ok(())
    }
}
