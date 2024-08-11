use dirtabase::archive::core::Triad;
use dirtabase::op::ctx::Context;
use dirtabase::storage;
use std::io::Result;

fn build_lua_5_4_7<P>(ctx: Context<P>) -> Result<Triad>
where
    P: AsRef<std::path::Path>,
{
    ctx.download(vec![
        "https://www.lua.org/ftp/lua-5.4.7.tar.gz".into(),
        "9fbf5e28ef86c69858f6d3d34eccc32e911c1a28b4120ff3e84aaa70cfbf1e30".into(),
    ])?
    .cmd_impure(vec!["tar zxf lua-5.4.7.tar.gz".into()])?
    .filter(vec!["^/lua-5.4.7".into()])?
    .prefix(vec!["lua-5.4.7".into(), "".into()])?
    .cmd_impure(vec!["make all test".into()])?
    .filter(vec!["src/lua$".into()])?
    .prefix(vec!["src".into(), "bin".into()])?
    .finish()
}

fn main() -> Result<()> {
    let store = storage::new_from_tempdir()?;
    let ctx = Context::new_from(&store, vec![]);
    let triad = build_lua_5_4_7(ctx)?;

    let sink = dirtabase::stream::osdir::sink("out");
    dirtabase::stream::archive::source(&store, triad, sink)?;

    Ok(())
}
