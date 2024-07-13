dirtabase
=========

Very WIP, do not even try to use!

Allows for manipulating immutable directories as objects that can pass between
processes. It's going to be the backbone of the `layover` package manager.

```bash
dirtabase \
  --ingest some/directory some/other/directory \
  --merge \
  --prefix some '' \
  --cmd 'find -type f | xargs md5sum > sums' \
  --filter sums \
  --export .

# --------------
# Equivalent to:
# --------------

# Bring external files into DB, printing a digest for each
dirtabase --ingest some/directory some/other/directory \
 \ # Consume multiple in-DB digests and produce one merged result
 \ # where later dirs override earlier dirs
 | dirtabase --merge \
 \ # Strip 'some' off the start of filenames within dirs
 | dirtabase --prefix some ''
 \ # Run a command on each top level directory that passes through
 | dirtabase --cmd 'find -type f | xargs md5sum > sums.txt'
 \ # Filter top level directories so each only has matching files
 | dirtabase --filter sums
 \ # Put files into the current OS directory (should just be ./sums.txt in this case)
 | dirtabase --export .
```

At each step, the interface is a stream of digests or other references
passing from one stage of processing to the next. That's the input and
output stream format of `dirtabase` Operators.

## URL format

```
scheme://engine-specific.url/stuff?including=engine_params#@label:path
```

Breaking down the parts:

 * `scheme`: Identifies which engine type to use
 * `fullpath`: Engine-specific configuration, including params
 * `ref`: May be a digest or a label
   * `digest` - "$format-$compression-$hexdigest"
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
