# Best Practices

This repo contains a crate called best-practices that contains some really
handy bits of code. It demonstrates how to use thiserror and anyhow to easily
have one set of Error types for any crate or app. It also demonstrates some
handy wrappers for handing `Option<PathBuf>` structopt options for input and
output to easily support both file and stdin/stdout I/O. This makes it easy
to construct command line apps that support pipelining (e.g. `echo "foo" | grep "foo" -`).
There's also a number of handy types for scanning directory trees, digesting
files and creating indexes of filesystem trees.

## Examples

This repo also contains some examples that demonstrate how to construct command
line apps the cleanest and easiest way.
