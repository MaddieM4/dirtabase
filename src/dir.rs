use serde::{Deserialize,Serialize};
use crate::digest::Digest;

#[derive(PartialEq,Debug,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Format {
    File,
    JSON,
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Compression {
    Plain,
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
pub struct Attr(String,String);
impl Attr {
    fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self(name.as_ref().into(), value.as_ref().into())
    }
}

#[derive(PartialEq,Debug,Serialize,Deserialize)]
pub struct ArchiveEntry {
    path: String,
    format: Format,
    compression: Compression,
    digest: Digest,
    attrs: Vec<Attr>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_roundtrip() -> serde_json::Result<()> {
        let archive: Vec<ArchiveEntry> = vec![
            ArchiveEntry {
                path: "foo/bar.txt".into(),
                format: Format::File,
                compression: Compression::Plain,
                digest: "some text".into(),
                attrs: vec![
                  Attr::new("unix_owner", "1000"),
                  Attr::new("unix_group", "1000"),
                  Attr::new("unix_flags", "0x777"),
                  Attr::new("frob_value", "absolutely frobnicated"),
                ]
            }
        ];

        let text: String = serde_json::to_string(&archive)?;
        assert_eq!(&text, r#"[{"path":"foo/bar.txt","format":"file","compression":"plain","digest":[185,79,111,18,92,121,227,165,255,170,130,111,88,76,16,213,42,218,102,158,103,98,5,27,130,107,85,119,109,5,174,210],"attrs":[["unix_owner","1000"],["unix_group","1000"],["unix_flags","0x777"],["frob_value","absolutely frobnicated"]]}]"#);

        let deserialized: Vec<ArchiveEntry> = serde_json::from_str(&text)?;
        assert_eq!(&deserialized, &archive);

        Ok(())
    }
}

// THE DEBATE ABOUT LABELS: To root archive or not to root archive?
//
// Approach 1: Have a distinct store for labels
// Pros:
//  * Easier to avoid contention between multiple writers (perf)
// Cons:
//  * Labels work in a weird custom way that doesn't share code
//  * Implementing attributes and compression is awkward
//  * Root archive format upgrades are awkward
//
// Approach 2: Have a top-level digest pointing to an archive
// Pros:
//  * Reusable code
//  * Consistency
//  * Conceptual simplicity
//  * Labels automatically get attributes and compression and format variance
//  * Root archive format upgrades are easy
// Cons:
//  * Contention management


/*
 * Process A and B are trying to write @A and @B respectively
 *
 * Notation for root nodes: {@A:1, @B:1} (label name and version)
 *
 * Start with an empty root node: {}
 * Process A generates {@A:1} and wants to promote it
 * Process B generates {@B:1} and wants to promote it
 *
 * Correct behavior:
 *  1. Process A reads root as {}
 *  2. Process A sets root to {@A:1}
 *  3. Process B reads root as {@A:1}
 *  4. Process B sets root to {@A:1, @B:1}
 *
 * Incorrect behavior:
 *  1. Process A reads root as {}
 *  2. Process B reads root as {}
 *  2. Process A sets root to {@A:1}
 *  4. Process B sets root to {@B:1}
 *
 * The answer: a mutex (MUTual EXclusion lock) that only allows 1 writer
 * to root at a time. The write action is incredibly fast. This is because
 * almost all of the work is writing to CAS, which is cheap to do without
 * contention. Reading, modifying, and writing the root node is only done
 * at the very end of the process, and is the critical section.
 *
 * Perf cost for larger critsec (read/modify/write):
 *  1. == ACQUIRE LOCK ==
 *  2. Get root digest/archive format
 *  3. Get buffer for root digest from CAS
 *  4. Parse bytes as an archive (archive format now in memory)
 *  5. Upsert the label we have a digest for into that archive
 *  6. Serialize archive to bytes
 *  7. Store archive bytes in new CAS buffer
 *  8. Set root to new digest/archive format
 *  9. == RELEASE LOCK ==
 *
 * Perf cost for smaller critsec (optimistic concurrency):
 *  1. Get root digest/archive format
 *  2. Get buffer for root digest from CAS
 *  3. Parse bytes as an archive (archive format now in memory)
 *  4. Upsert the label we have a digest for into that archive
 *  5. Serialize archive to bytes
 *  6. Store archive bytes in new CAS buffer
 *  7. == ACQUIRE LOCK ==
 *  8. Atomic update to digest/archive format (MAY REPORT FAILURE)
 *  9. == RELEASE LOCK ==
 *  10. If Step 8 failed, return to Step 1 (2 if using compare-and-swap)
 *
 *
 * .dirtabase_db/
 *   - root -> ['protobuf_archive', 'ffffffffffffff']
 *   - cas/
 *     - 37878787878878...
 *     - 12345678990909...
 *     - abcdefabcdefab... { @A: "37878787878878", @B: "12345678990909" }
 *     - ffffffffffffff... { @B: "12345678990909" }
 *     - eeeeeeeeeeeeee... { @C: "12345678990909" }
 */
