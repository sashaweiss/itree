extern crate clap;
extern crate itree;

use clap::{App, Arg};

use itree::{color, options, term, tree};

use std::io::{self, Write};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

fn main() {
    let (options, no_render) = parse_args();

    if no_render {
        tree::Tree::new_with_options(options);
        return;
    }

    let (sx, rx) = channel();

    thread::spawn(move || {
        // Only start loading dialog if it takes more than 300ms to build the tree
        if let Err(_) = rx.recv_timeout(Duration::from_millis(300)) {
            let mut stdout = io::stdout();

            write!(stdout, "Building tree").unwrap();
            stdout.flush().unwrap();
            loop {
                // Print a dot every 1000ms
                if let Ok(_) = rx.recv_timeout(Duration::from_millis(1000)) {
                    break;
                }
                write!(stdout, ".").unwrap();
                stdout.flush().unwrap();
            }
            writeln!(stdout, "done!").unwrap();
        }
    });

    let mut t = tree::Tree::new_with_options(options);
    sx.send(()).unwrap();

    term::navigate(&mut t);
}

fn string_to_color(cs: &str) -> Box<color::Color> {
    match cs {
        "black" => Box::new(color::Black),
        "blue" => Box::new(color::Blue),
        "cyan" => Box::new(color::Cyan),
        "green" => Box::new(color::Green),
        "magenta" => Box::new(color::Magenta),
        "red" => Box::new(color::Red),
        "white" => Box::new(color::White),
        "yellow" => Box::new(color::Yellow),
        "lightblack" => Box::new(color::LightBlack),
        "lightblue" => Box::new(color::LightBlue),
        "lightcyan" => Box::new(color::LightCyan),
        "lightgreen" => Box::new(color::LightGreen),
        "lightmagenta" => Box::new(color::LightMagenta),
        "lightred" => Box::new(color::LightRed),
        "lightwhite" => Box::new(color::LightWhite),
        "lightyellow" => Box::new(color::LightYellow),
        _ => panic!("unrecognized color string"),
    }
}

fn parse_args() -> (options::TreeOptions<String>, bool) {
    let matches = App::new("itree")
        .about("An interactive version of the `tree` utility")
        .author("Sasha Weiss <sasha@sashaweiss.coffee>")
        .arg(
            Arg::with_name("no_render")
                .long("no-render")
                .help("Do not display the tree - just build it. Intended for benchmarking"),
        )
        .arg(
            Arg::with_name("max_level")
                .short("L")
                .long("max-level")
                .help("Max recursion level")
                .takes_value(true)
                .validator(|s| s.parse::<usize>().map(|_| {}).map_err(|e| format!("{}", e))),
        )
        .arg(
            Arg::with_name("follow_links")
                .short("l")
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
                .multiple(true)
                .validator(|s| options::validate_ignore(&s)),
        )
        .arg(
            Arg::with_name("bg_color")
                .short("c")
                .long("bg-color")
                .help("The background color to highlight the focused file. Blue by default")
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
            Arg::with_name("fg_color")
                .short("f")
                .long("fg-color")
                .help("The foreground color to use to draw the tree. White by default")
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
                .index(1)
                .help("The directory at which to start the tree"),
        )
        .get_matches();

    let mut options = options::TreeOptions::new(".".to_owned());
    options
        .max_depth(
            matches
                .value_of("max_level")
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
        .use_git_exclude(matches.is_present("use_git_exclude"))
        .fg_color(string_to_color(
            matches.value_of("fg_color").unwrap_or("white"),
        ))
        .bg_color(string_to_color(
            matches.value_of("bg_color").unwrap_or("blue"),
        ));

    if let Some(files) = matches.values_of("custom_ignore") {
        for file in files {
            options.add_custom_ignore(&format!("!{}", file));
        }
    }

    if let Some(root) = matches.value_of("root") {
        options.root(root.to_owned());
    }

    (options, matches.is_present("no_render"))
}
