extern crate clap;
extern crate ignore;
extern crate indextree;
extern crate termion;

use clap::{App, Arg};

mod fs;
mod term;
mod tree;

fn main() {
    let (options, color) = parse_args();

    match tree::Tree::new_with_options(options) {
        Ok(mut t) => term::navigate(&mut t, &(*color)),
        Err(e) => eprintln!("{:?}", e),
    };
}

fn string_to_color(cs: &str) -> Box<termion::color::Color> {
    match cs {
        "black" => Box::new(termion::color::Black),
        "blue" => Box::new(termion::color::Blue),
        "cyan" => Box::new(termion::color::Cyan),
        "green" => Box::new(termion::color::Green),
        "magenta" => Box::new(termion::color::Magenta),
        "red" => Box::new(termion::color::Red),
        "white" => Box::new(termion::color::White),
        "yellow" => Box::new(termion::color::Yellow),
        "lightblack" => Box::new(termion::color::LightBlack),
        "lightblue" => Box::new(termion::color::LightBlue),
        "lightcyan" => Box::new(termion::color::LightCyan),
        "lightgreen" => Box::new(termion::color::LightGreen),
        "lightmagenta" => Box::new(termion::color::LightMagenta),
        "lightred" => Box::new(termion::color::LightRed),
        "lightwhite" => Box::new(termion::color::LightWhite),
        "lightyellow" => Box::new(termion::color::LightYellow),
        _ => panic!("unrecognized color string"),
    }
}

fn parse_args() -> (tree::TreeOptions<String>, Box<termion::color::Color>) {
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
            Arg::with_name("use_git_exclude")
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
        .use_git_exclude(matches.is_present("use_git_exclude"));

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
            .map(|s| string_to_color(s))
            .unwrap_or(Box::new(termion::color::Blue)),
    )
}
