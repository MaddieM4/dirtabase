use dirtabase::prelude::*;

// TODO:
//  * Bundle SDL and other deps
//  * XDG application
//  * Set up in a consistent place in home dir

fn main() -> Result<()> {
    let db = DB::new("./.dirtabase_db")?;
    let mut log = Logger::new_real();
    ctx(&db, &mut log)
        .download(
            "http://www.retroguru.com/gianas-return/gianas-return-v.latest-linux.tar.gz",
            "515af14bc425dac9b5368792b287ebbb3b973e435be80676b4db9789ef71b4c1",
        )?
        .cmd_impure("tar xf gianas-return-v.latest-linux.tar.gz && rm *.tar.gz")?
        .export("out")?;
    Ok(())
}
