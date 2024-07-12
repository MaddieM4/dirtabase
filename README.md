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
output stream format of `dirtabase` operators.

## Reference format

```
scheme://engine-specific.url/stuff?including=engine_params#@label:path
```

Breaking down the parts:

 * `scheme`: Identifies which engine type to use
 * `fullpath`: Engine-specific configuration, including params
 * `ref`: May be a digest or a label
   * `digest` - just literally 64 chars of hexified SHA256 hash
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
