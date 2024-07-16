pub use engines::{Simple, Store};
pub use types::*;

fn main() {
    let mut engine = Simple::new("./.dirtabase_db").expect("Failed to init engine");
    let hw = engine
        .store("Hello, world!")
        .expect("Should store resource");
    println!("Stored object: {:?}", hw.digest);

    let archive = Archive {
        format: Format::JSON,
        compression: Compression::Plain,
        entries: vec![ArchiveEntry {
            path: "some/file.txt".into(),
            spec: Spec {
                format: Format::File,
                compression: Compression::Plain,
                digest: hw.digest.clone(),
            },
            attrs: vec![
                Attr::new("unix_owner", "1000"),
                Attr::new("unix_group", "1000"),
                Attr::new("unix_flags", "0x777"),
                Attr::new("frob_value", "absolutely frobnicated"),
            ],
        }],
    };
    let arch_rsc = engine.store(archive.to_buffer()).expect("Store @example archive");
    println!("Stored @example archive: {:?}", arch_rsc.digest);

    engine.set_label("example", &Spec {
        format: archive.format,
        compression: archive.compression,
        digest: arch_rsc.digest.clone(),
    }).expect("Failed to store rootdata");

    engine.set_label("example2", &Spec {
        format: archive.format,
        compression: archive.compression,
        digest: arch_rsc.digest,
    }).expect("Failed to store rootdata");
}
