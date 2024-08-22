use dirtabase::prelude::*;

/*

I'm using this as a testing ground for full userspace packaging before
inevitably moving some of the logic to Layover as a separate project.

You can try it by adding this to your ~/.profile:

```
# Layover support
export PATH="$HOME/.layover/bin:$PATH"
export XDG_DATA_DIRS="$XDG_DATA_DIRS:$HOME/.layover"
```

You may need to log out and log in to see the effects.

Now just run `cargo run --example lua $HOME/.layover` and you should have
Lua 5.4.7 available on your system, with an XDG desktop entry. Neat, huh?

You can run this without any system installation with just
`cargo run --example lua` and no further arguments. It builds to ./out.
*/

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
        .rename("src/lua$", "bin/lua")?
        .cmd_impure("convert doc/logo.gif doc/logo.png")?
        .rename("doc/logo.png$", "icons/lua.png")?
        .filter("^(bin|icons)")?
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
