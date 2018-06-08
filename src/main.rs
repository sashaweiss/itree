extern crate clap;
extern crate ignore;
extern crate indextree;
extern crate termion;

use clap::{App, Arg};

mod fs;
mod term;
mod tree;

fn main() {
    let matches = App::new("rusty-tree")
        .about("An interactive version of the `tree` utility")
        .arg(
            Arg::with_name("max_depth")
                .short("d")
                .long("max-depth")
                .help("Max recursion depth")
                .takes_value(true)
                .validator(|s| s.parse::<usize>().map(|_| {}).map_err(|e| format!("{}", e))),
        )
        .arg(
            Arg::with_name("follow_links")
                .short("L")
                .long("follow-links")
                .help("Follow links"),
        )
        .arg(
            Arg::with_name("max_filesize")
                .short("M")
                .long("max-filesize")
                .help("Max file size to include")
                .takes_value(true)
                .validator(|s| s.parse::<u64>().map(|_| {}).map_err(|e| format!("{}", e))),
        )
        .arg(
            Arg::with_name("hidden")
                .short("H")
                .long("hidden")
                .help("Include hidden files"),
        )
        .arg(
            Arg::with_name("ignore")
                .short("i")
                .long("ignore-files")
                .help("Do not respect `.ignore` files"),
        )
        .arg(
            Arg::with_name("git_global")
                .short("G")
                .long("git-global")
                .help("Do not respect the global `.gitignore` file, if present"),
        )
        .arg(
            Arg::with_name("git_ignore")
                .short("g")
                .long("git-ignore")
                .help("Do not respect `.gitignore` files"),
        )
        .arg(
            Arg::with_name("git_exclude")
                .short("x")
                .long("git-exclude")
                .help("Do not respect `.git/info/exclude` files"),
        )
        .arg(
            Arg::with_name("custom_ignore")
                .short("I")
                .long("ignore")
                .help("Do not respect `.git/info/exclude` files")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("root")
                .last(true)
                .help("The directory at which to start the tree"),
        )
        .get_matches();

    let mut options = tree::TreeOptions::new();
    options
        .max_depth(
            matches
                .value_of("max_depth")
                .map(|s| s.parse::<usize>().unwrap()),
        )
        .follow_links(matches.is_present("follow_links"))
        .max_filesize(
            matches
                .value_of("max_filesize")
                .map(|s| s.parse::<u64>().unwrap()),
        )
        .hidden(!matches.is_present("hidden"))
        .ignore(!matches.is_present("ignore"))
        .git_global(!matches.is_present("git_global"))
        .git_ignore(!matches.is_present("git_ignore"))
        .git_exclude(!matches.is_present("git_exclude"));

    if let Some(files) = matches.values_of("custom_ignore") {
        for file in files {
            options.add_custom_ignore(&file);
        }
    }

    let root = match matches.value_of("root") {
        Some(r) => r,
        None => ".",
    };

    let mut t = tree::Tree::new_with_options(&root, options);
    term::navigate(&mut t);
}
