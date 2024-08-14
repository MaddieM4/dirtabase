use dirtabase::build::prelude::*;

fn main() -> Result<()> {
    let db = DB::new(".dirtabase_db")?;
    let d = Ark::scan("./fixture")?.import(&db)?;
    println!("Archive import completed.\nDigest: {}.", d.to_hex());
    Ok(())
}
