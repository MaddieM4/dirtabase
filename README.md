dirtabase
=========

Very WIP, do not even try to use!

Allows for manipulating immutable directories as objects that can pass between
processes. It's going to be the backbone of the `layover` package manager.

```bash
# Run this command in this repo!
$ dirtabase \
  --import src fixture \
  --merge \
  --prefix '' misc/ \
  --cmd-impure 'find misc -type f | xargs md5sum > sums' \
  --filter '^/sums' \
  --export out
```

Here's what it'd output:

```
--- Import ---
json-plain-c6ef2ee35a879ac0d1ae28296fd22040b8de5a32f7cc6e26eead1678b8243745
json-plain-d6467585a5b63a42945759efd8c8a21dfd701470253339477407653e48a3643a
--- Merge ---
json-plain-97e19e493248433a639a916620fe6849998058f354cff414e339ce52b8155685
--- Prefix ---
json-plain-90f16a5cd6faf1bc5c3455dad18efddf27295d7127136d6c52991524e268ea30
--- CmdImpure ---
--- [find misc -type f | xargs md5sum > sums] ---
json-plain-fc0f1953c55e18c006896ac591e866eea53d46e0ccd1671a023530388ab854ab
--- Filter ---
json-plain-150e580c81762725735a4a4936727401ff8e0b567c0d00b34dc572b0e073eff9
--- Export ---
```

And you can poke around at the directory you just made!

```bash
$ ls out
sums

$ cat out
e2af4a9feae56aae3cdb746746fb3e53  misc/cli.rs
50cd68ca08155c7edf90a27f8b96d3de  misc/archive/mod.rs
26d38af62510f25049629bbe0ed034ca  misc/archive/core.rs
8992012555033d4f1ef994c76369df7a  misc/archive/normalize.rs
4b3ede399b76bb628dd8126a0b21a76a  misc/archive/api.rs
a93e3e781bd142655aa4330620c94574  misc/op/mod.rs
c37128567009ebaa7b3bc79cf18ad110  misc/op/cmd.rs
66fe2517f5c22e17e830a25602e110b3  misc/digest.rs
dfe710d9f603c791106f3f2472d56f74  misc/stream/mod.rs
ee9904318c3b80ee901e0ed23a1d7441  misc/stream/core.rs
9787349243c44d8340fe6ffaa06f63e3  misc/stream/osdir.rs
144d465f78e3bbd33d46f0585c514eec  misc/stream/archive.rs
241f545cf7c26e72cfdfb2839683d348  misc/stream/debug.rs
429227c7d2bf39a268455c141e128eb0  misc/attr.rs
54d6fb85bf9f831812f93c21ea914f3c  misc/main.rs
9d358d667fe119ed3a8a98faeb0de40b  misc/dir1/dir2/nested.txt
167fce463d52ee9fc7058a7887fab2ec  misc/label.rs
df89b4c7ad822e8823269320117e4e96  misc/storage/mod.rs
c2ad5a560c4bd82ebed21abfd444a5f8  misc/storage/core.rs
17880d64921d32ef670d412fb05cf418  misc/storage/traits.rs
eb6f71ea9c0627e30d358601dd2eb658  misc/storage/simple.rs
c2333d995e4dbacab98f9fa37a1201a9  misc/file_at_root.txt
5d4848dbb6f29792b773a6d78a84f06c  misc/lib.rs
```

At each step, the interface is a stream of digests or other references
passing from one stage of processing to the next. That's the input and
output stream format of `dirtabase` Operators.

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

## URL format

```
scheme://engine-specific.url/stuff?including=engine_params#@ref:path
```

Breaking down the parts:

 * `scheme`: Identifies which engine type to use
 * `fullpath`: Engine-specific configuration, including params
 * `ref`: May be a digest or a label
   * `triad` - "$format-$compression-$hexdigest"
   * `label` - a human-readable mutable label starting in `@`
 * `path`: navigates from `ref` to a file or directory within it

Fully canonical URLs are rare, and may be constructed by taking a shorter URL
and canonicalizing according to the following rules:

 * If scheme and fullpath are not provided:
   * URLs that start with `#...` are expanded to `default:///#...`
   * URLs that start with `@...` are expanded to `default:///#@...`
   * URLs that start with anything else are expanded to `file://.../#:`
 * `ref`, if missing, is presumed to be `@root`
 * `path`, if missing, is presumed to be `:.`
 * `default:///` is converted into a default engine config, which can be customized with environment variables.

This means that for common cases, you don't need to do much work to specify
inputs or outputs unambiguously - there is a deterministic conversion to canon form.
This is very ergonomic and convenient. References passing between processing
stages are always in canon form.

## Storage model

There's a few tiers of conceptual heirarchy that I'll discuss from highest-level to lowest.

### Operators and Pipelines

An Operator is essentially a function that acts on a few _parameters_ (specific to the Operator) and a stream of incoming references (which may be for any storage engine). It produces a stream of outgoing references. Each reference is a URL in the format described above, and _usually_ describes an entire immutable directory tree rather than a single file.

A Pipeline is responsible for automatically instantiating storage engines for Operators, funneling data between them, etc. Using `dirtabase` as a CLI tool (rather than just a library) is intrinsically about building a Pipeline of Operators.

### Archives

An Archive is an immutable directory tree. Any changes to itself or its child nodes technically counts as a different Archive with its own hash digest. In pseudocode, you can imagine an Archive being structured something like this:

```
/foo.txt -> FILE abcdef123456...
/bar -> JSON_ARCHIVE 98989898989898...
...
/baz/quux.rs -> FILE 12312312312...
```

Archives can reference other archives, including them as subdirectories. They can also directly reference files at any level of depth. You can think of it logically like a stream, where referencing another Archive is like a prefixed "include" directive, and later entries override previous entries. This means the "merge" Operator is equivalent to just concatenating every Archive that goes into the merge, in order.

For these reasons, an Archive can technically be expressed unambiguously while being full of redundancy and conflicts, as there's always a simple deterministic way to interpret them. However, messily-structured Archives can penalize the performance of a pipeline, so when possible, it's good to simplify an Archive before outputting it. A correctly written Operator or traversal function will happily consume a messy Archive, but always produce a clean one.

### Labels and Root

A `dirtabase` DB will happily store "anonymous" Archives, especially as intermediate processing results. Sometimes, though, you want to store an Archive under a human-friendly name in the database, and protect it from garbage collection (GC is a necessity whenever you're working with immutable data).

This is what Labels are for. Labels start with a `@`, so for example, `@layover-upstream` might be a plausible label used by the Layover package manager (time will tell). These are easy to store to, and read from, with `dirtabase` commands. The list of Labels, and which store contents they point to (usually an Archive each), is itself an Archive, with the special constraint that every entry has a path that starts with `@` and prohibits certain special characters.

This "all the Labels" archive needs to be promoted as official for the whole storage backend _somehow_, which is why each backend has a teeny tiny piece of "rootdata" consisting of a `(format, compression, digest)` tuple. This `rootdata` can be updated atomically with compare-and-swap, so modifying the root archive is achieved by the following recipe, which is a form of optimistic concurrency:

1. Load and parse the current root archive
2. Modify as you need
3. Store modified version (candidate) in engine storage
4. Attempt to update the `rootdata` to point to your new root archive.
  * Only works if `rootdata` is what you found during Step 1.
  * If it works, you're done!
  * If it fails, go back to Step 1 and try again.

Why optimistic concurrency? Because we support a variety of storage engines, many of them on-disk, with a variety of underlying semantics and deadlock potential. That's simpler to do right and fast by using an optimistic concurrency model to absolutely minimize the critical section. Now, it does have potential perf penalties with a lot of competing writers that are all living in Retry Hell, but in expected use dynamics, it's a _lot_ easier to tolerate than long-lasting locks.

### Engine Storage (CAS)

From the ground up, `dirtabase` is designed to be able to load and save data from multiple storage backends. As an example, you'll often use the OSDir backend to import Archives into database format, or export it back out again. This system is intentionally extensible, to allow for things like working purely in program memory or emitting Docker images.

The underlying storage model, in general, is a content-addressed store, or "CAS" for short. This is just a simple map of `digest => contents` such that `sha256sum(contents) == digest`. This has a few consequences that probably aren't surprising if you think them through:

 1. You can only compute the digest by knowing the contents first.
 2. This means you can't reference some other piece of content without it being set in stone.
 3. And that means loops, including self-references, are impossible.
 4. If you try to store identical files, they won't take up extra space. They'll have the same digest and be stored in the same slot. Resource-level deduplication is intrinsic to the design.

Archives, then, are just _stored buffers that happen to be in an Archive format._ You need to store all the constituent file buffers first that you want to point to, or at least hash them. Only after constructing the child nodes can you know the hash of the Archive, and store _that_.

The default storage backend is going to have some kind of name like `simpledb` and a file structure like:

```
.dirtabase_db
 - root -> ['protobuf_archive', 'ffffffffffffff']
 - cas/
   - 37878787878878...
   - 12345678990909...
   - abcdefabcdefab... { @A: "37878787878878", @B: "12345678990909" }
   - ffffffffffffff... { @B: "12345678990909" }
   - eeeeeeeeeeeeee... { @C: "12345678990909" }
```

Where every resource is stored in a corresponding file in the `cas` directory. It's noteworthy that there are no OS directories under `cas`, since Archives are - again - just files in an appropriate format.
