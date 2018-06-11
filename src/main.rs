extern crate clap;
extern crate ignore;
extern crate indextree;
extern crate termion;

use clap::{App, Arg};

mod fs;
mod term;
mod tree;

fn main() {
    let (options, color_string) = parse_args();

    match tree::Tree::new_with_options(options) {
        Ok(mut t) => {
            match color_string.as_str() {
                "black" => term::navigate(&mut t, &termion::color::Black),
                "blue" => term::navigate(&mut t, &termion::color::Blue),
                "cyan" => term::navigate(&mut t, &termion::color::Cyan),
                "green" => term::navigate(&mut t, &termion::color::Green),
                "magenta" => term::navigate(&mut t, &termion::color::Magenta),
                "red" => term::navigate(&mut t, &termion::color::Red),
                "white" => term::navigate(&mut t, &termion::color::White),
                "yellow" => term::navigate(&mut t, &termion::color::Yellow),
                "lightblack" => term::navigate(&mut t, &termion::color::LightBlack),
                "lightblue" => term::navigate(&mut t, &termion::color::LightBlue),
                "lightcyan" => term::navigate(&mut t, &termion::color::LightCyan),
                "lightgreen" => term::navigate(&mut t, &termion::color::LightGreen),
                "lightmagenta" => term::navigate(&mut t, &termion::color::LightMagenta),
                "lightred" => term::navigate(&mut t, &termion::color::LightRed),
                "lightwhite" => term::navigate(&mut t, &termion::color::LightWhite),
                "lightyellow" => term::navigate(&mut t, &termion::color::LightYellow),
                _ => panic!("unrecognized color string"),
            };
        }
        Err(e) => eprintln!("{:?}", e),
    };
}

fn parse_args() -> (tree::TreeOptions<String>, String) {
    let matches = App::new("rusty-tree")
        .about("An interactive version of the `tree` utility")
        .author("Sasha Weiss <sasha@sashaweiss.coffee>")
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
                .long("max-filesize")
                .help("Max file size to include")
                .takes_value(true)
                .validator(|s| s.parse::<u64>().map(|_| {}).map_err(|e| format!("{}", e))),
        )
        .arg(
            Arg::with_name("hidden")
                .long("hidden")
                .help("Include hidden files"),
        )
        .arg(
            Arg::with_name("use_ignore")
                .long("no-ignore")
                .help("Do not respect `.[git]ignore` files"),
        )
        .arg(
            Arg::with_name("no_git_exclude")
                .long("no-exclude")
                .help("Do not respect `.git/info/exclude` files"),
        )
        .arg(
            Arg::with_name("custom_ignore")
                .short("I")
                .long("ignore")
                .help("Specify an additional path to ignore")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("color")
                .short("c")
                .long("color")
                .help("The color to highlight the focused file. Blue by default")
                .takes_value(true)
                .possible_values(&[
                    "black",
                    "blue",
                    "cyan",
                    "green",
                    "magenta",
                    "red",
                    "white",
                    "yellow",
                    "lightblack",
                    "lightblue",
                    "lightcyan",
                    "lightgreen",
                    "lightmagenta",
                    "lightred",
                    "lightwhite",
                    "lightyellow",
                ]),
        )
        .arg(
            Arg::with_name("root")
                .last(true)
                .help("The directory at which to start the tree"),
        )
        .get_matches();

    let mut options = tree::TreeOptions::new(".".to_owned());
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
        .hidden(matches.is_present("hidden"))
        .use_ignore(matches.is_present("use_ignore"))
        .no_git_exclude(matches.is_present("no_git_exclude"));

    if let Some(files) = matches.values_of("custom_ignore") {
        for file in files {
            options.add_custom_ignore(&format!("!{}", file));
        }
    }

    if let Some(root) = matches.value_of("root") {
        options.root(root.to_owned());
    }

    (
        options,
        matches
            .value_of("color")
            .map(|s| s.to_owned())
            .unwrap_or("blue".to_owned()),
    )
}
