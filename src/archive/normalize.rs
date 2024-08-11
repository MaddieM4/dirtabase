//! Archive normalization logic.

use crate::archive::core::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

/// Simplify an Archive without changing how it would convert to a tree.
///
/// This isn't just for removing duplicate entries for the same path, although
/// of course it does that. It actually puts a lot of work (see test suite size)
/// into producing a series of entries that can be easily and naively digested
/// by `dirtabase::stream::osdir::sink` even while setting file permissions as
/// we go.
///
/// The rules are:
///
///  * No duplicate entries with the same path. In ties, the last item wins.
///  * All files come before all dirs
///      (because they autovivify their parent dirs with writer permissions)
///  * Directories must be sorted from most to least nested
///      (imagine trying to give a/b/c to root after ceding a/b to root!)
///
///  See <https://github.com/MaddieM4/dirtabase/issues/6> for more details.
pub fn normalize(ar: &Archive) -> Archive {
    let mut overrides_applied: Vec<(PathBuf, Entry)> = ar
        .iter()
        .map(|e| {
            let path = match e {
                Entry::File { path, .. } => path,
                Entry::Dir { path, .. } => path,
            }
            .clone();
            (path, e.clone())
        })
        .collect::<HashMap<PathBuf, Entry>>()
        .into_iter()
        .collect();

    // Sort that handles partitioning and directory nesting at the same time
    overrides_applied.sort_by_key(|(p, e)| {
        (
            // Sort primarily by file-ness
            match e {
                Entry::File { .. } => 0,
                Entry::Dir { .. } => 1,
            },
            // Secondarily by path in reverse order
            Reverse(p.clone()),
        )
    });
    overrides_applied.into_iter().map(|(_, e)| e).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;
    use std::collections::HashSet;

    #[derive(PartialEq, Debug)]
    struct File(Attrs, Digest);

    #[derive(PartialEq, Debug)]
    struct Dir(Attrs, HashMap<String, Tree>);

    #[derive(PartialEq, Debug)]
    enum Tree {
        F(File),
        D(Dir),
    }

    fn dir<const N: usize>(
        name: &str,
        attrs: Attrs,
        entries: [(String, Tree); N],
    ) -> (String, Tree) {
        (name.to_owned(), Tree::D(Dir(attrs, entries.into())))
    }

    fn file(name: &str, attrs: Attrs, content: &str) -> (String, Tree) {
        (name.to_owned(), Tree::F(File(attrs, content.into())))
    }

    fn traverse<'a>(root: &'a mut Dir, p: &std::path::Path) -> &'a mut Dir {
        let mut cursor = root;
        for c in p.components() {
            if let std::path::Component::Normal(s) = c {
                let s = String::from(s.to_str().unwrap());
                let child = cursor
                    .1
                    .entry(s)
                    .or_insert_with(|| Tree::D(Dir(at! {}, HashMap::new())));
                cursor = match child {
                    Tree::F(_) => panic!("That wasn't a directory!"),
                    Tree::D(d) => d,
                };
            }
        }
        cursor
    }

    fn archive_to_tree(ar: &Archive) -> Dir {
        let mut root = Dir(at! {}, HashMap::new());
        for entry in ar {
            match entry {
                Entry::File {
                    path,
                    attrs,
                    compression: _,
                    digest,
                } => {
                    let path: &std::path::Path = path.as_ref();
                    let dir = match path.parent() {
                        Some(p) => traverse(&mut root, p),
                        None => &mut root,
                    };
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    dir.1
                        .insert(filename.into(), Tree::F(File(attrs.clone(), *digest)));
                }
                Entry::Dir { path, attrs } => {
                    let dir = traverse(&mut root, path.as_ref());
                    dir.0 = attrs.clone();
                }
            }
        }
        root
    }

    #[test]
    fn tree_examples() {
        assert_eq!(archive_to_tree(&vec![]), Dir(at! {}, HashMap::new()),);

        assert_eq!(
            archive_to_tree(&vec![Entry::File {
                path: "foo/bar/baz.txt".into(),
                attrs: at! {A1=>"Sauce"},
                compression: Compression::Plain,
                digest: "Some content".into(),
            }]),
            Dir(
                at! {},
                HashMap::from([dir(
                    "foo",
                    at! {},
                    [dir(
                        "bar",
                        at! {},
                        [file("baz.txt", at! {A1=>"Sauce"}, "Some content"),]
                    ),]
                ),])
            ),
        );

        assert_eq!(
            archive_to_tree(&vec![
                Entry::File {
                    path: "foo/bar/baz.txt".into(),
                    attrs: at! {A1=>"Sauce"},
                    compression: Compression::Plain,
                    digest: "Some content".into(),
                },
                Entry::Dir {
                    path: "foo/xyz".into(),
                    attrs: at! { NAME=>"xyz", SECOND_VAR=>"2" },
                },
                Entry::Dir {
                    path: "foo/xyz".into(),
                    attrs: at! { NAME=>"xyz-revisited", THIRD_VAR=>"3" },
                },
                Entry::Dir {
                    path: "/".into(),
                    attrs: at! { ROOT=>"is me" }
                },
                Entry::File {
                    path: "foo/bar/baz.txt".into(),
                    attrs: at! {A1=>"Drip"},
                    compression: Compression::Plain,
                    digest: "Other content".into(),
                },
                Entry::Dir {
                    path: "foo".into(),
                    attrs: at! { I_AM=>"Foo!" },
                },
            ]),
            Dir(
                at! { ROOT=>"is me" },
                HashMap::from([dir(
                    "foo",
                    at! { I_AM=>"Foo!" },
                    [
                        dir(
                            "bar",
                            at! {},
                            [file("baz.txt", at! {A1=>"Drip"}, "Other content"),]
                        ),
                        dir("xyz", at! { NAME=>"xyz-revisited", THIRD_VAR=>"3" }, []),
                    ]
                ),])
            ),
        );
    }

    #[test]
    fn normalize_proptests() {
        fn examine(msg: &str, ar: Archive) {
            let normalized = normalize(&ar);
            assert_eq!(
                archive_to_tree(&normalized),
                archive_to_tree(&ar),
                "Does not result in the same tree: {}",
                msg
            );

            let mut paths_seen = HashSet::<std::path::PathBuf>::new();
            for entry in &normalized {
                let path = match entry {
                    Entry::File { path, .. } => path,
                    Entry::Dir { path, .. } => path,
                }
                .clone();
                assert!(
                    !paths_seen.contains(&path),
                    "Path {} appears multiple times: {}",
                    path.display(),
                    msg
                );
                paths_seen.insert(path);
            }

            // All files precede all directories
            let mut in_files_section = true;
            for entry in &normalized {
                match entry {
                    Entry::File { .. } => assert!(
                        in_files_section,
                        "File appeared after the end of the file section: {}",
                        msg
                    ),
                    Entry::Dir { .. } => in_files_section = false,
                }
            }

            // Directories go from most nested to least nested
            let mut dirs_seen = HashSet::<std::path::PathBuf>::new();
            for entry in &normalized {
                if let Entry::Dir { path, .. } = entry {
                    let path = path.clone();
                    for possible_base in &dirs_seen {
                        assert!(
                            !path.starts_with(possible_base),
                            "Dir {:?} must come before dir {:?}: {}",
                            &path,
                            &possible_base,
                            &msg
                        )
                    }
                    dirs_seen.insert(path);
                }
            }
        }

        examine("Empty archive", vec![]);
        examine(
            "Directory appears twice",
            vec![
                Entry::Dir {
                    path: "foo/xyz".into(),
                    attrs: at! { NAME=>"xyz", SECOND_VAR=>"2" },
                },
                Entry::Dir {
                    path: "foo/xyz".into(),
                    attrs: at! { NAME=>"xyz-revisited", THIRD_VAR=>"3" },
                },
            ],
        );
        examine(
            "Files and directories intermixed when naively sorted",
            vec![
                Entry::Dir {
                    path: "abc".into(),
                    attrs: at! {},
                },
                Entry::File {
                    path: "def".into(),
                    attrs: at! {},
                    compression: Compression::Plain,
                    digest: "contents".into(),
                },
                Entry::Dir {
                    path: "ghi".into(),
                    attrs: at! {},
                },
            ],
        );
        examine(
            "Nested order and naive sort order conflict",
            vec![
                Entry::Dir {
                    path: "a".into(),
                    attrs: at! {},
                },
                Entry::Dir {
                    path: "a/b".into(),
                    attrs: at! {},
                },
                Entry::Dir {
                    path: "a/b/c".into(),
                    attrs: at! {},
                },
            ],
        );
    }
}
