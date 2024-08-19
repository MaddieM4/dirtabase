pub const REPRODUCIBLE_URL: &str = "https://gist.githubusercontent.com/MaddieM4/92f0719922db5fbd60a12d762deca9ae/raw/37a4fe4d300b6a88913a808095fd52c1c356030a/reproducible.txt";
pub const REPRODUCIBLE_DIGEST: &str =
    "460f3d82bf451fbebd1958fe4714e2a82a6570dda19e0d6f39cd7504adca6088";

// Todo: move into prefix op
#[cfg(test)]
fn prefix_ark<C>(ark: arkive::Ark<C>, prefix: &str) -> arkive::Ark<C> {
    use arkive::ToIPR;
    let (p, a, c) = ark.decompose();
    let p: Vec<arkive::IPR> = p
        .iter()
        .map(|ipr| (prefix.to_owned() + "/" + ipr.as_ref()).to_ipr())
        .collect();
    arkive::Ark::compose(std::rc::Rc::new(p), a, c)
}

#[cfg(test)]
pub fn fixture_digest() -> arkive::Digest {
    let db = arkive::DB::new_temp().expect("Temp DB");
    let fixture_ark = prefix_ark(
        arkive::Ark::scan("fixture").expect("Scan fixture dir"),
        "fixture",
    );
    assert_eq!(fixture_ark.len(), 4);
    let digest = fixture_ark.import(&db).expect("Imported to temp DB");
    digest
}
