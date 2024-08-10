use crate::archive::core::*;
use crate::archive::normalize::normalize;
use crate::storage::simple::SimpleStorage;
use regex::Regex;
use std::io::{Cursor, Read as _, Result};

// How on earth do we want to interact with a Storage?

pub fn archive_encode(ar: &Archive, _f: ArchiveFormat, _c: Compression) -> Result<Vec<u8>> {
    serde_json::to_vec(ar).map_err(|e| std::io::Error::other(e))
}

pub fn archive_decode(bytes: Vec<u8>, _f: ArchiveFormat, _c: Compression) -> Result<Archive> {
    serde_json::from_slice(bytes.as_ref()).map_err(|e| std::io::Error::other(e))
}

pub fn write_archive(
    ar: &Archive,
    f: ArchiveFormat,
    c: Compression,
    store: &SimpleStorage,
) -> Result<Digest> {
    // Turn `ar` into `bytes: Vec<u8>`
    let bytes = archive_encode(ar, f, c)?;

    // Make a Cursor on bytes
    let curs = Cursor::new(bytes);

    // Use that in cas.write()
    store.cas().write(curs)
}

pub fn read_archive(
    f: ArchiveFormat,
    c: Compression,
    digest: &Digest,
    store: &SimpleStorage,
) -> Result<Archive> {
    let mut bytes: Vec<u8> = vec![];
    store
        .cas()
        .read(digest)?
        .ok_or::<std::io::Error>(std::io::ErrorKind::NotFound.into())?
        .read_to_end(&mut bytes)?;
    archive_decode(bytes, f, c)
}

// TODO: move to utils
fn path_str(p: impl AsRef<std::path::Path>) -> String {
    p.as_ref()
        .to_str()
        .expect("Could not convert path to string")
        .into()
}

pub fn filter(ar: Archive, re: &Regex) -> Archive {
    ar.into_iter()
        .filter(|entry| {
            let s: String = path_str(match entry {
                Entry::Dir { path, .. } => path,
                Entry::File { path, .. } => path,
            });
            re.is_match(&s)
        })
        .collect()
}

pub fn replace(ar: Archive, re: &Regex, replacement: &str) -> Archive {
    fn replace_path(p: PathBuf, re: &Regex, replacement: &str) -> PathBuf {
        let haystack = p.to_str().expect("Could not convert path to str");
        String::from(re.replace(haystack, replacement)).into()
    }
    let replaced = ar.into_iter()
        .map(|entry| match entry {
            Entry::Dir { path, attrs } => Entry::Dir {
                path: replace_path(path, re, replacement),
                attrs: attrs,
            },
            Entry::File {
                path,
                attrs,
                compression,
                digest,
            } => Entry::File {
                path: replace_path(path, re, replacement),
                attrs: attrs,
                compression: compression,
                digest: digest,
            },
        })
        .collect();

    normalize(&replaced)
}

pub fn merge(ars: &[Archive]) -> Archive {
    normalize(&ars.concat())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn round_trip_encoding() {
        let before: Archive = vec![Entry::File {
            path: "/hello/world.txt".into(),
            compression: Compression::Plain,
            digest: "some contents".into(),
            attrs: Attrs::new().set("MIME", "text/plain"),
        }];

        let bytes = archive_encode(&before, ArchiveFormat::JSON, Compression::Plain)
            .expect("Should not fail to serialize");
        assert!(bytes.len() > 0);

        let after = archive_decode(bytes, ArchiveFormat::JSON, Compression::Plain)
            .expect("Should not fail to deserialize");
        assert_eq!(after, before);
    }

    #[test]
    fn test_filter() {
        let ar: Archive = vec![
            Entry::Dir {
                path: "/foo/bar".into(),
                attrs: Attrs::new(),
            },
            Entry::File {
                path: "/nomatch".into(),
                attrs: Attrs::new(),
                compression: Compression::Plain,
                digest: "xyz".into(),
            },
            Entry::File {
                path: "/match/me/foo.py".into(),
                attrs: Attrs::new(),
                compression: Compression::Plain,
                digest: "xyz".into(),
            },
            Entry::Dir {
                path: "/fail".into(),
                attrs: Attrs::new(),
            },
        ];
        assert_eq!(
            filter(ar, &Regex::new("foo").unwrap()),
            vec![
                Entry::Dir {
                    path: "/foo/bar".into(),
                    attrs: Attrs::new(),
                },
                Entry::File {
                    path: "/match/me/foo.py".into(),
                    attrs: Attrs::new(),
                    compression: Compression::Plain,
                    digest: "xyz".into(),
                },
            ]
        )
    }

    #[test]
    fn test_replace() {
        let ar: Archive = vec![
            Entry::Dir {
                path: "/foo/bar".into(),
                attrs: Attrs::new(),
            },
            Entry::File {
                path: "/nomatch".into(),
                attrs: Attrs::new(),
                compression: Compression::Plain,
                digest: "xyz".into(),
            },
            Entry::File {
                path: "/match/me/foo.py".into(),
                attrs: Attrs::new(),
                compression: Compression::Plain,
                digest: "xyz".into(),
            },
        ];
        assert_eq!(
            replace(ar, &Regex::new("match").unwrap(), "matcha"),
            vec![
                Entry::File {
                    path: "/nomatcha".into(),
                    attrs: Attrs::new(),
                    compression: Compression::Plain,
                    digest: "xyz".into(),
                },
                Entry::File {
                    path: "/matcha/me/foo.py".into(),
                    attrs: Attrs::new(),
                    compression: Compression::Plain,
                    digest: "xyz".into(),
                },
                Entry::Dir {
                    path: "/foo/bar".into(),
                    attrs: Attrs::new(),
                },
            ]
        )
    }

    #[test]
    fn test_merge() {
        let ar1: Archive = vec![
            Entry::Dir {
                path: "/uniq/1".into(),
                attrs: at! { AR=>"1" },
            },
            Entry::Dir {
                path: "/common".into(),
                attrs: at! { AR=>"1" },
            },
        ];
        let ar2: Archive = vec![
            Entry::Dir {
                path: "/uniq/2".into(),
                attrs: at! { AR=>"2" },
            },
            Entry::Dir {
                path: "/common".into(),
                attrs: at! { AR=>"2" },
            },
        ];
        let ar3: Archive = vec![
            Entry::Dir {
                path: "/uniq/3".into(),
                attrs: at! { AR=>"3" },
            },
            Entry::Dir {
                path: "/common".into(),
                attrs: at! { AR=>"3" },
            },
        ];
        assert_eq!(
            merge(&[ar1, ar2, ar3]),
            vec![
                Entry::Dir {
                    path: "/uniq/3".into(),
                    attrs: at! { AR=>"3" }
                },
                Entry::Dir {
                    path: "/uniq/2".into(),
                    attrs: at! { AR=>"2" }
                },
                Entry::Dir {
                    path: "/uniq/1".into(),
                    attrs: at! { AR=>"1" }
                },
                Entry::Dir {
                    path: "/common".into(),
                    attrs: at! { AR=>"3" }
                },
            ]
        );
    }
}
