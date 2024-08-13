use dirtabase::archive::core::Triad;
use dirtabase::logger::Logger;
use dirtabase::op::helpers::{ctx, Context};
use dirtabase::storage;
use std::io::Result;

fn build_lua_5_4_7<P>(ctx: Context<P>) -> Result<Triad>
where
    P: AsRef<std::path::Path>,
{
    ctx.download(
        "https://www.lua.org/ftp/lua-5.4.7.tar.gz",
        "9fbf5e28ef86c69858f6d3d34eccc32e911c1a28b4120ff3e84aaa70cfbf1e30",
    )?
    .cmd_impure("tar zxf lua-5.4.7.tar.gz")?
    .filter("^/lua-5.4.7")?
    .prefix("lua-5.4.7", "")?
    .cmd_impure("make all test")?
    .filter("src/lua$")?
    .prefix("src", "bin")?
    .finish()
}

fn main() -> Result<()> {
    let store = storage::new_from_tempdir()?;
    let mut log = Logger::default();
    let triad = build_lua_5_4_7(ctx(&store, &mut log))?;

    let sink = dirtabase::stream::osdir::sink("out");
    dirtabase::stream::archive::source(&store, triad, sink)?;

    Ok(())
}
