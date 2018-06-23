extern crate clap;
extern crate itree;

use clap::{App, Arg};

use itree::{color, options, render, term, tree};

use std::io::{self, Write};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

enum RenderMethod {
    JustSummary,
    NoInteractive,
    FullInteractive,
}

fn main() {
    let (fs_opts, rd_opts, rm) = parse_args();

    let mut t = build_tree_loading(fs_opts);
    let mut render = render::TreeRender::new(&mut t, rd_opts);

    match rm {
        RenderMethod::JustSummary => {
            println!("\n{}", render.tree.summary());
        }
        RenderMethod::NoInteractive => {
            print!("{}", render);
        }
        RenderMethod::FullInteractive => {
            term::navigate(&mut render);
        }
    }
}

fn build_tree_loading(opts: options::FsOptions<String>) -> tree::Tree {
    let (sx, rx) = channel();
    thread::spawn(move || {
        // Only start loading dialog if it takes more than 300ms to build the tree
        if let Err(_) = rx.recv_timeout(Duration::from_millis(300)) {
            let mut stderr = io::stderr();

            write!(stderr, "Building tree").unwrap();
            stderr.flush().unwrap();
            loop {
                // Print a dot every 1000ms
                if let Ok(_) = rx.recv_timeout(Duration::from_millis(1000)) {
                    break;
                }
                write!(stderr, ".").unwrap();
                stderr.flush().unwrap();
            }
            writeln!(stderr, "done!").unwrap();
        }
    });

    let t = tree::Tree::new_with_options(opts);
    sx.send(()).unwrap();

    t
}

fn colors() -> &'static [&'static str] {
    &[
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
    ]
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

fn parse_args() -> (
    options::FsOptions<String>,
    options::RenderOptions,
    RenderMethod,
) {
    let matches = App::new("itree")
        .about("An interactive version of the `tree` utility")
        .author("Sasha Weiss <sasha@sashaweiss.coffee>")
        .version("0.3.0")
        .args(&[
            no_interact_arg(),
            quiet_arg(),
            only_dirs_arg(),
            level_arg(),
            link_arg(),
            filesize_arg(),
            hidden_arg(),
            no_ignore_arg(),
            no_exclude_arg(),
            custom_ignore_arg(),
            bg_color_arg(),
            fg_color_arg(),
            root_arg(),
        ])
        .get_matches();

    let mut fs_options = options::FsOptions::new(".".to_owned());
    fs_options
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
        .only_dirs(matches.is_present("only_dirs"))
        .no_ignore(matches.is_present("no_ignore"))
        .no_git_exclude(matches.is_present("no_git_exclude"));

    if let Some(files) = matches.values_of("custom_ignore") {
        for file in files {
            fs_options.add_custom_ignore(&format!("!{}", file));
        }
    }

    if let Some(root) = matches.value_of("root") {
        fs_options.root(root.to_owned());
    }

    let mut rd_options = options::RenderOptions::new();
    rd_options
        .fg_color(string_to_color(
            matches.value_of("fg_color").unwrap_or("white"),
        ))
        .bg_color(string_to_color(
            matches.value_of("bg_color").unwrap_or("blue"),
        ));

    let rm: RenderMethod;
    if matches.is_present("quiet") {
        rm = RenderMethod::JustSummary;
    } else if matches.is_present("no_interact") {
        rm = RenderMethod::NoInteractive;
    } else {
        rm = RenderMethod::FullInteractive;
    }

    (fs_options, rd_options, rm)
}

fn no_interact_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("no_interact")
        .long("no-interact")
        .help("Do not enter interactive mode - just print the tree and summary information.")
        .conflicts_with("quiet")
}

fn quiet_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("quiet")
        .long("quiet")
        .help("Do not render the tree - just build it and print summary information.")
        .conflicts_with("no_interact")
}

fn only_dirs_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("only_dirs")
        .long("only-dirs")
        .help("List directories only")
}

fn level_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("max_level")
        .short("L")
        .long("max-level")
        .help("Max recursion level")
        .takes_value(true)
        .validator(|s| s.parse::<usize>().map(|_| {}).map_err(|e| format!("{}", e)))
}

fn link_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("follow_links")
        .short("l")
        .long("follow-links")
        .help("Follow links")
}

fn filesize_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("max_filesize")
        .long("max-filesize")
        .help("Max file size to include")
        .takes_value(true)
        .validator(|s| s.parse::<u64>().map(|_| {}).map_err(|e| format!("{}", e)))
}

fn hidden_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("hidden")
        .long("hidden")
        .help("Include hidden files")
}

fn no_ignore_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("no_ignore")
        .long("no-ignore")
        .help("Do not respect `.[git]ignore` files")
}

fn no_exclude_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("no_git_exclude")
        .long("no-exclude")
        .help("Do not respect `.git/info/exclude` files")
}

fn custom_ignore_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("custom_ignore")
        .short("I")
        .long("ignore")
        .help("Specify an additional path to ignore")
        .takes_value(true)
        .number_of_values(1)
        .multiple(true)
        .validator(|s| options::validate_ignore(&s))
}

fn bg_color_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("bg_color")
        .short("c")
        .long("bg-color")
        .help("The background color to highlight the focused file. Blue by default")
        .takes_value(true)
        .possible_values(colors())
}

fn fg_color_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("fg_color")
        .short("f")
        .long("fg-color")
        .help("The foreground color to use to draw the tree. White by default")
        .takes_value(true)
        .possible_values(colors())
}

fn root_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("root")
        .index(1)
        .help("The directory at which to start the tree")
}
