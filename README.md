# itree
`itree` is an interactively navigable version of the Linux utility `tree`, built in Rust. It aims to extend the functionality of `tree`, providing an intuitive view of a directory's structure in a manageable, interactively navigable command-line interface.

Whereas `tree` can be overwhelming to use on large directories, `itree` allows you to inspect a filesystem in an intuitive fashion, at your own pace. It also respects gitignore rules, allowing you to focus on the files you're most interested in!

`itree` relies on the FS walker used by [ripgrep](https://github.com/BurntSushi/ripgrep/tree/master/ignore), bringing you the usefulness of `tree` with as little overhead/slowdown as possible!

`tree`             |  `itree`
:-------------------------:|:-------------------------:
![Running `tree` from the ~/.rustup directory][rustup_tree_gif]  |  ![Running `itree` from the ~/.rustup directory][rustup_itree_gif]
![Running `tree` from this project's directory][itree_tree_gif]  |  ![Running `itree` from this project's directory][itree_itree_gif]

[rustup_itree_gif]: https://media.giphy.com/media/7Jq6jStr9wlVU79gzR/giphy.gif
[itree_itree_gif]: https://media.giphy.com/media/9xaSG1BHu6GmAZlJTS/giphy.gif
[rustup_tree_gif]: https://media.giphy.com/media/4Q06kUFcUcyeo0IVRz/giphy.gif
[itree_tree_gif]: https://media.giphy.com/media/fdXPaasUE6eKnCF6gs/giphy.gif

## Installation
### Using `brew`
`itree` is available via Homebrew! Simply run:
```
$ brew install sashaweiss/projects/itree
```

### From source
To build from source, first make sure you have Rust and `cargo` installed! (If not, then install via [Rustup](https://rustup.rs/).) Then:
```
$ git clone https://github.com/sashaweiss/itree
$ cd itree
$ cargo install
```

## Usage
Running `itree` will start an interactive CLI. Use `itree -h` to see a full list of configurations and UI options!

* Use the arrow keys to move around, as makes sense visually: `Up` and `Down` move between files in the same directory level, while `Left` and `Right` move one level higher and lower in the directory tree, respectively.
* Use `f` to fold/unfold a directory.
* Use `q`, `Ctrl-C`, or `Esc` to exit.

More commands to come! (E.g. deleting, moving, renaming files.)

## My to-do list

### High priority
* Benchmark running from `~`, vs. `tree` and `rg --files`
* Write more comprehensive documentation of source code

### Medium priority
* Implement optional information displays that `tree` offers, e.g. `tree -h`
* Add commands for interacting with files under the cursor
* Add command for `cd`-ing to the folder the cursor is currently in
* Figure out why `parse_args` panics when given stdin input

### Low priority
* Integration tests for command-line args
