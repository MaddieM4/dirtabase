use dirtabase::prelude::*;

fn main() -> Result<()> {
    let db = DB::new("./.dirtabase_db")?;
    let mut log = Logger::new_real();

    // Default to ./out, but can be overridden to $HOME/.layover easily
    let args: Vec<_> = std::env::args().skip(1).collect();
    let layover_dir = if args.is_empty() {
        let local_out = std::env::current_dir()?.join("out");
        local_out.to_string_lossy().to_string()
    } else {
        args[0].clone()
    };

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
        // Stack on the desktop file
        .import(".", ["examples"])?
        .cmd_impure(
            "m4 '-DLAYOVER_DIR=".to_owned() + &layover_dir + "' examples/lua.desktop > lua.desktop",
        )?
        .filter("^lua")?
        .prefix("applications")?
        .merge()?
        .export(layover_dir)?;

    Ok(())
}
