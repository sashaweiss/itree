* Integration tests for command-line args
* Render symlinks as "<name> -> <dest>"
* Optionally render information about each node, like file size (`tree -h`)
* Figure out why `parse_args` panics when given stdin input
* `Tree::new_with_options` doesn't need to give back a result - add a validator to the ignore globs
* Benchmark running from `~`, vs. `tree` and `rg --files`.
* Build a homebrew formula
