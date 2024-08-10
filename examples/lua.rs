use dirtabase::op::ctx::{Context, DEFAULT_ENCODING};
use dirtabase::storage::traits::Storage;
use dirtabase::storage::simple::storage;
use dirtabase::archive::core::Triad;

use tempfile::tempdir;
use std::io::Result;

fn build_lua_5_4_7<S>(ctx: Context<S>) -> Result<Triad>
where
    S: Storage,
{
    ctx.download_impure(vec!["https://www.lua.org/ftp/lua-5.4.7.tar.gz".into()])?
        .cmd_impure(vec!["tar zxf lua-5.4.7.tar.gz".into()])?
        .filter(vec!["^/lua-5.4.7".into()])?
        .prefix(vec!["lua-5.4.7".into(), "".into()])?
        .cmd_impure(vec!["make all test".into()])?
        .filter(vec!["src/lua$".into()])?
        .prefix(vec!["src".into(), "bin".into()])?
        .finish()
}

fn main() -> Result<()> {
    let store_dir = tempdir()?;
    let store = storage(store_dir.path())?;
    let ctx = Context::new(&store, DEFAULT_ENCODING);
    let triad = build_lua_5_4_7(ctx)?;

    let sink = dirtabase::stream::osdir::sink("out");
    dirtabase::stream::archive::source(&store, triad, sink)?;

    Ok(())
}
