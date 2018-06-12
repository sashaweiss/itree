# rusty-tree
Interactive and navigable version of `tree`

### To-do list
Future work around the project.

#### Short-term
* Render symlinks as "<name> -> <dest>"
* Optionally render information about each node, like file size (`tree -h`)
* `Tree::new_with_options` doesn't need to give back a result - add a validator to the ignore globs

#### Long-term
* Benchmark running from `~`, vs. `tree` and `rg --files`.
* Figure out why `parse_args` panics when given stdin input
* Integration tests for command-line args
* Build a homebrew formula
