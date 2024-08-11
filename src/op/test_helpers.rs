pub use crate::archive::core::Triad;

pub fn fixture_triad() -> Triad {
    let txt = r#"[{"archive":"json"},"plain","90d0cf810af44cbf7a5d24a9cca8bad6e3724606b28880890b8639da8ee6f7e4"]"#;
    serde_json::from_str(&txt).expect("fixture_triad: failed to parse")
}

pub fn random_triad() -> Triad {
    let hexdigest = {
        use rand::RngCore;
        let mut random_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut random_bytes);
        hex::encode(random_bytes)
    };

    let txt = str::replace(r#"[{"archive":"json"},"plain","HEX"]"#, "HEX", &hexdigest);
    serde_json::from_str(&txt).expect("fixture_triad: failed to parse")
}

pub fn random_triads<const N: usize>() -> [Triad; N] {
    [(); N]
        .into_iter()
        .map(|_| random_triad())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
