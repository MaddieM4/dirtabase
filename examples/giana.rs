use dirtabase::prelude::*;

fn main() -> Result<()> {
    let db = DB::new("./.dirtabase_db")?;
    let mut log = Logger::new_real();
    ctx(&db, &mut log)
        .download_impure("https://gist.githubusercontent.com/MaddieM4/92f0719922db5fbd60a12d762deca9ae/raw/37a4fe4d300b6a88913a808095fd52c1c356030a/reproducible.txt")?
        .export("out")?;
    Ok(())
}
