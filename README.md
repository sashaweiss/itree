# itree
`itree` is an interactively navigable version of the Linux utility `tree`, built in Rust. It aims to extend the functionality of `tree`, providing an intuitive view of a directory's structure in a manageable, interactively navigable command-line interface.

Whereas `tree` can be overwhelming to use on large directories, `itree` allows you to inspect a filesystem in an intuitive fashion, at your own pace. It also respects gitignore rules and supports folding, allowing you to focus on the files you're most interested in!

`itree` relies on the FS walker used by [ripgrep](https://github.com/BurntSushi/ripgrep), bringing you the usefulness of `tree` with as little overhead/slowdown as possible! (See [benchmarks](#benchmarks) below for comparisons.)

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

### Using `cargo`
`itree` is also available on crates.io! Simply run:
```
$ cargo install itree
```

### From source
To build from source, first make sure you have Rust and `cargo` installed. (If not, then install via [Rustup](https://rustup.rs/).) Then:
```
$ git clone https://github.com/sashaweiss/itree
$ cd itree
$ cargo install
```

## Usage
Running `itree` will start an interactive CLI. Use `itree --help` to see a full list of configurations and UI options!

* Use the arrow keys to move around, as makes sense visually: `Up` and `Down` move between files in the same directory level, while `Left` and `Right` move one level higher and lower in the directory tree, respectively.
  * `itree` also supports Vim keybindings - `h`, `j`, `k`, and `l` can be used instead of the arrow keys.
* Use `f` to fold/unfold a directory.
* Use `q`, `Ctrl-C`, or `Esc` to exit.

## Benchmarks
Below are tables comparing the performance of `itree` to that of `tree` as well as to `ripgrep` (from which `itree` gets its filesystem iterator).

Tl;dr: `itree` is fast - much faster than `tree`, and not a huge slowdown over pure `ripgrep`! (Mostly thanks to [BurntSushi](https://github.com/burntsushi)'s awesome `ignore` [crate](https://github.com/BurntSushi/ripgrep/tree/master/ignore)).

### Methodology
I used [hyperfine](https://github.com/sharkdp/hyperfine) for benchmarking - specifically, the command:
```
$ hyperfine --warmup 2 <CMD> --show-output
```
where `<CMD>` was filled in with the first column of the below tables. `--show-output` was used to avoid suppressing the output of each command, since printing/rendering is an important part of what `tree` and `itree` do. `--warmup 2` caused each command to run twice before being measured, to potentially warm up caches.

### Results
Results shown are the mean and standard deviation reported by `hyperfine`. Each is the result of at least 10 measurements.

* The first row shows how long a user would wait for `itree` to display its UI.
* The second row shows how long `itree` takes to exactly emulate the behavior of `tree`.
* The third row shows how long `tree` takes to draw a directory structure.
* The final row shows how long `ripgrep` takes to silently scan the directory structure. Since `ripgrep` and `itree` use the same filesystem iterator, this represents a baseline for computing `itree`'s overhead.

| Command | μ ± σ (run from my `$HOME`) | μ ± σ (run in this repo) |
|:---|:---:|:---:|
| `itree --no-ignore --no-exclude --no-render` | **2.953s** ± 0.070s | **0.012s** ± 0.002s |
| `itree --no-ignore --no-exclude --no-interact` | 3.511s ± 0.043s | 0.031s ± 0.010s |
| `tree` | 15.005s ± 4.891s | 0.043s ± 0.014s |
| `rg --no-ignore --files --quiet` | 1.373s ± 0.051s | 0.010s ± 0.014s |

## Future work
* Write more comprehensive documentation of source code.
* Implement functionality similar to `tree -h`.
* Add commands for interacting with files under the cursor.
* Add command for `cd`-ing to the folder the cursor is currently in.
* Figure out why argument parsing panics when given stdin input.
