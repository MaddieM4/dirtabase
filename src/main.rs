pub use engines::{Simple, Store};
pub use types::*;

fn main() {
    let mut engine = Simple::new("./.dirtabase_db").expect("Failed to init engine");
    let hw = engine
        .store("Hello, world!")
        .expect("Should store resource");
    println!("Stored object: {:?}", hw.digest);

    let archive = vec![ArchiveEntry {
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
    }];
    let arch_bytes = serde_json::to_vec(&archive).unwrap();
    let arch_rsc = engine.store(arch_bytes).expect("Store arch_bytes");
    println!("Stored archive: {:?}", arch_rsc.digest);

    let root = vec![ArchiveEntry {
        path: "@example".into(),
        spec: Spec {
            format: Format::JSON,
            compression: Compression::Plain,
            digest: arch_rsc.digest.clone(),
        },
        attrs: vec![],
    }];
    let root_bytes = serde_json::to_vec(&root).unwrap();
    let root_rsc = engine.store(root_bytes).expect("Store root_bytes");
    println!("Stored root archive: {:?}", root_rsc.digest);

    engine.replace_root(None, Some(Spec {
        format: Format::JSON,
        compression: Compression::Plain,
        digest: root_rsc.digest.clone(),
    })).expect("Failed to store rootdata");
}
