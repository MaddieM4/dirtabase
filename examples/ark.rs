use dirtabase::build::prelude::*;

fn main() -> Result<()> {
    let db = DB::new("db")?;
    let ark = Ark::scan("./fixture")?.import_files(&db)?;
    let txt = serde_json::to_string(&ark)?;
    let d = Digest::from(&txt);
    std::fs::write(db.join("cas").join(d.to_hex()), &txt)?;

    Ok(())
}
