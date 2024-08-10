pub mod ctx;
use crate::op::ctx::Context;
use crate::archive::core::Triad;
use crate::storage::simple::SimpleStorage;
use std::io::Result;

// TODO: Multi-backend interaction

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    Empty,
    Import,
    Export,
    Merge,
    Filter,
    Replace,
    Prefix,
    CmdImpure,
}

pub fn perform(
    op: Op,
    store: &SimpleStorage,
    triads: Vec<Triad>,
    params: Vec<String>,
) -> Result<Vec<Triad>> {
    match op {
        Op::Empty => Ok(Context::new_from(store, triads).empty()?.triads),
        Op::Import => Ok(Context::new_from(store, triads).import(params)?.triads),
        Op::Export => Ok(Context::new_from(store, triads).export(params)?.triads),
        Op::Merge => Ok(Context::new_from(store, triads).merge()?.triads),
        Op::Filter => Ok(Context::new_from(store, triads).filter(params)?.triads),
        Op::Replace => Ok(Context::new_from(store, triads).replace(params)?.triads),
        Op::Prefix => Ok(Context::new_from(store, triads).prefix(params)?.triads),
        Op::CmdImpure => Ok(Context::new_from(store, triads).cmd_impure(params)?.triads),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::archive::core::{Attrs, Compression, TriadFormat};
    use crate::digest::Digest;
    use crate::storage::simple::storage;
    use crate::stream::core::Sink;
    use indoc::indoc;
    use tempfile::tempdir;

    fn fixture_triad() -> Result<Triad> {
        let dir = tempdir()?;
        let store = storage(dir.path())?;
        let sink = crate::stream::archive::sink(&store);

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
    fn merge() -> Result<()> {
        let store_dir = tempdir()?;
        let store = storage(store_dir.path())?;
        use crate::stream::archive::sink;

        let triad_dbg = crate::stream::debug::source(sink(&store))?;
        let triad_fix = crate::stream::osdir::source("./fixture", sink(&store))?;

        let merged = perform(Op::Merge, &store, vec![triad_dbg, triad_fix], vec![])?;
        assert_eq!(merged.len(), 1);
        let txt = crate::stream::archive::source(&store, merged[0], crate::stream::debug::sink())?;
        assert_eq!(
            txt,
            indoc! {"
          FILE /some/dir/hello.txt
            Length: 17
            AnotherAttr: for example purposes
          FILE /file_at_root.txt
            Length: 37
          FILE /dir1/dir2/nested.txt
            Length: 41
          DIR /dir1/dir2
          DIR /dir1
          DIR /a/directory
            Foo: Bar
        "}
        );
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
        assert!(!out.path().join("file_at_root.txt").exists());
        Ok(())
    }

    #[test]
    fn replace() -> Result<()> {
        let store_dir = tempdir()?;
        let store = storage(store_dir.path())?;
        use crate::stream::archive::sink;

        let triad_dbg = crate::stream::debug::source(sink(&store))?;
        let triad_fix = crate::stream::osdir::source("./fixture", sink(&store))?;

        // Should only affect last triad
        let output = perform(
            Op::Replace,
            &store,
            vec![triad_dbg, triad_fix],
            vec!["root".into(), "boot".into()],
        )?;
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], triad_dbg);

        // Let's read out the transformed item from the top of the stack
        let txt = crate::stream::archive::source(&store, output[1], crate::stream::debug::sink())?;
        assert_eq!(
            txt,
            indoc! {"
          FILE /file_at_boot.txt
            Length: 37
          FILE /dir1/dir2/nested.txt
            Length: 41
          DIR /dir1/dir2
          DIR /dir1
        "}
        );
        Ok(())
    }

    #[test]
    fn prefix() -> Result<()> {
        let store_dir = tempdir()?;
        let store = storage(store_dir.path())?;
        use crate::stream::archive::sink;

        let triad_dbg = crate::stream::debug::source(sink(&store))?;
        let triad_fix = crate::stream::osdir::source("./fixture", sink(&store))?;

        // Should only affect last triad
        let output = perform(
            Op::Prefix,
            &store,
            vec![triad_dbg, triad_fix],
            vec!["dir".into(), "folder".into()],
        )?;
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], triad_dbg);

        // Let's read out the transformed item from the top of the stack
        let txt = crate::stream::archive::source(&store, output[1], crate::stream::debug::sink())?;
        assert_eq!(
            txt,
            indoc! {"
          FILE /folder1/dir2/nested.txt
            Length: 41
          FILE /file_at_root.txt
            Length: 37
          DIR /folder1/dir2
          DIR /folder1
        "}
        );

        // No replacement possible
        let output = perform(
            Op::Prefix,
            &store,
            vec![triad_fix],
            vec!["dir2".into(), "folder2".into()],
        )?;
        let txt = crate::stream::archive::source(&store, output[0], crate::stream::debug::sink())?;
        assert_eq!(
            txt,
            indoc! {"
          FILE /file_at_root.txt
            Length: 37
          FILE /dir1/dir2/nested.txt
            Length: 41
          DIR /dir1/dir2
          DIR /dir1
        "}
        );

        // User provided redundant symbols
        let output = perform(
            Op::Prefix,
            &store,
            vec![triad_fix],
            vec!["^/d".into(), "/c".into()],
        )?;
        let txt = crate::stream::archive::source(&store, output[0], crate::stream::debug::sink())?;
        assert_eq!(
            txt,
            indoc! {"
          FILE /file_at_root.txt
            Length: 37
          FILE /cir1/dir2/nested.txt
            Length: 41
          DIR /cir1/dir2
          DIR /cir1
        "}
        );

        Ok(())
    }
}
