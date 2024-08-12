use crate::archive::core::Archive;
pub use crate::archive::core::Triad;
use crate::digest::Digest;
pub use indoc::indoc;

fn triad(hexdigest: impl AsRef<str>) -> Triad {
    let txt = str::replace(
        r#"[{"archive":"json"},"plain","HEX"]"#,
        "HEX",
        hexdigest.as_ref(),
    );
    serde_json::from_str(&txt).expect("fixture_triad: failed to parse")
}

pub fn empty_triad() -> Triad {
    let empty_archive: Archive = vec![];
    let d = Digest::from(serde_json::to_string(&empty_archive).unwrap());
    triad(d.to_hex())
}

pub fn fixture_triad() -> Triad {
    triad("90d0cf810af44cbf7a5d24a9cca8bad6e3724606b28880890b8639da8ee6f7e4")
}

pub fn random_triad() -> Triad {
    triad({
        use rand::RngCore;
        let mut random_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut random_bytes);
        hex::encode(random_bytes)
    })
}

pub fn random_triads<const N: usize>() -> [Triad; N] {
    [(); N]
        .into_iter()
        .map(|_| random_triad())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
