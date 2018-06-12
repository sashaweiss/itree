# rusty-tree
`rusty-tree` is an interactively navigable version of the Linux utility `tree`, built in Rust. It aims to extend the functionality of `tree`, providing an intuitive view of a directory's structure in a manageable, interactively navigable command-line interface.

Whereas `tree` can be overwhelming to use on large directories, `rusty-tree` allows you to inspect a filesystem in an intuitive fashion, at your own pace. It also respects gitignore rules, allowing you to focus on the files you're most interested in.

`rusty-tree` relies on the FS walker used by [ripgrep](https://github.com/BurntSushi/ripgrep/tree/master/ignore), bringing you the usefulness of `tree` with as little overhead/slowdown as possible!

`tree`             |  `rusty-tree`
:-------------------------:|:-------------------------:
![Running `tree` from the ~/.rustup directory][rustup_tree_gif]  |  ![Running `rusty-tree` from the ~/.rustup directory][rustup_rt_gif]
![Running `tree` from this project's directory][rt_tree_gif]  |  ![Running `rusty-tree` from this project's directory][rt_rt_gif]

[rustup_rt_gif]: https://media.giphy.com/media/oOnUBSE5gL45B1zk5K/giphy.gif
[rt_rt_gif]: https://media.giphy.com/media/9JeJTMYkjcwGmB2XdO/giphy.gif
[rustup_tree_gif]: https://media.giphy.com/media/kVgETL09kI8pi24RfR/giphy.gif
[rt_tree_gif]: https://media.giphy.com/media/1d5QqzHOvEHrnfnH6P/giphy.gif

## Installation
`rusty-tree` is not yet on any package managers, and so currently must be built from source. Make sure you have Rust and `cargo` installed! (If not, then install via [Rustup](https://rustup.rs/).) Then:
```
$ git clone https://github.com/sashaweiss/rusty-tree
$ cd rusty-tree
$ cargo install
```

Package managers and precompiled binaries to come!

## Usage
Running `rusty-tree` will start an interactive CLI.

* Use the arrow keys to move around, as makes sense visually: `Up` and `Down` move between files in the same directory level, while `Left` and `Right` move one level higher and lower in the directory tree, respectively.
* Use `q`, `Ctrl-C`, or `Esc` to exit.

More commands to come! (E.g. deleting, moving, renaming files.)

### My to-do list

#### Short-term
* Render symlinks as "\<name\> -> \<dest\>"
* Optionally render information about each node, like file size (`tree -h`)
* `Tree::new_with_options` doesn't need to give back a result - add a validator to the ignore globs
* Add a loading bar for when the tree is being constructed

#### Long-term
* Benchmark running from `~`, vs. `tree` and `rg --files`.
* Figure out why `parse_args` panics when given stdin input
* Integration tests for command-line args
* Build a homebrew formula
* Add commands for interacting with files under the cursor
