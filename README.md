dirtabase
=========

A build system using [Arkive](https://github.com/MaddieM4/arkive), but
providing a higher level list of verbs (operations), including downloads and
command execution, with caching. It's going to be the backbone of the `layover`
package manager.

```bash
# Run this command in this repo!
$ dirtabase \
  --import . src fixture \
  --merge \
  --prefix misc \
  --cmd-impure 'find misc -type f | xargs md5sum > sums' \
  --filter '^sums' \
  --export out
```

Here's what it'd output (this will probably be less chatty in the future):

```
================================================================
Import
================================================================
 + Can cache? false
 + Is in cache? false
dd45aedac81fb5e08f594bee978c9c6bd74b758f4f458ccd4fe250d271dcf171
8c958951d9f61be6a7b1ec48611710efc3d12ee71f3dc6ac34251afe4a95378e
================================================================
Merge
================================================================
 + Can cache? true
 + Is in cache? true
fe4462adb040549b5e632c4962e9ddfd98cd7f710949a50c137a351547eb170d
================================================================
Prefix
================================================================
 + Can cache? true
 + Is in cache? true
f5587f960dc28e8753f8558f61567cef5ed820ba9a87792d64162aed5fe9f4e0
================================================================
CmdImpure
================================================================
 + Can cache? false
 + Is in cache? false
--- [find misc -type f | xargs md5sum > sums] ---
20b1c125cbbc550603a3bbf5e6dec21802a656bf1f2d23b11011430d94f86b3b
================================================================
Filter
================================================================
 + Can cache? true
 + Is in cache? true
56b34c726418366b10db4cffe4285e04d47fd6f8161b1cda7a4bdc1a302c83e5
================================================================
Export
================================================================
 + Can cache? false
 + Is in cache? false
```

And you can poke around at the directory you just made!

```bash
$ ls out
sums

$ cat out
c2333d995e4dbacab98f9fa37a1201a9  misc/fixture/file_at_root.txt
9d358d667fe119ed3a8a98faeb0de40b  misc/fixture/dir1/dir2/nested.txt
1dba60d0147ca0771b3e2553de7fb2f2  misc/src/context.rs
9156988bafe609194d7aca6adf8a3e70  misc/src/doc.rs
cc255b333228984a0bbccbcf1a16f1d0  misc/src/cli.rs
f18205c6a9877b2e6cb757cfeb266dfc  misc/src/test_tools.rs
9c8a8227ccef3ec678df0723e7621bd8  misc/src/op.rs
74d1290949aca1cd5bc4d3b4128ae99d  misc/src/prelude.rs
b330c35e6816a7895e0d202458d591c0  misc/src/behavior.rs
799a951d84acaad174313a340c730dc6  misc/src/lib.rs
5d6c6c5d29506c037eecc4611afb18ec  misc/src/main.rs
f1bbacd456d6e7695ed60d7c0d6d1901  misc/src/logger.rs
```

At each step, the interface is a stream of archives passing from one stage of
processing to the next. That's the input and output stream format of
`dirtabase` Operators. The cache can actually pick back up after uncacheable
steps, because each archive has a full hash of its contents - we can recognize
when we've stumbled back into familiar territory.

The biggest missing pieces at this point are sandboxed (pure) commands, and
Layover building on top of this tech.

# Contributing

This repo is equipped to use [devenv.sh](https://devenv.sh/), which is [pretty
easy](https://devenv.sh/getting-started/#installation) to get set up. It also
integrates nicely with [direnv](https://devenv.sh/automatic-shell-activation/).

```bash
# These commands should work after setup!
direnv test
direnv shell
```

I'm going to also set up building this as a Nix package/flake later.
