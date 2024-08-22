use dirtabase::prelude::*;

fn main() -> Result<()> {
    let db = DB::new("./.dirtabase_db")?;
    let mut log = Logger::new_real();
    ctx(&db, &mut log)
        .download(
            "https://www.lua.org/ftp/lua-5.4.7.tar.gz",
            "97f31885f6f80b9049396e496f8fde9d2ea5b774784fbc5c16bb42d6fd640642",
        )?
        .cmd_impure("tar zxf *.tar.gz && rm *.tar.gz")?
        .rename("^lua-5.4.7/", "")?
        .cmd_impure("make all test")?
        .filter("src/lua$")?
        .rename("src", "bin")?
        .export("out")?;

    Ok(())
}
