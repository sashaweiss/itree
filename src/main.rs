extern crate clap;
extern crate itree;

mod args;

use args::*;
use itree::{options, render, term, tree};

use std::io::{self, Write};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

const VERSION: &str = "0.3.1";

fn main() {
    let (fs_opts, rd_opts, rm) = parse_args(VERSION);

    let mut t = build_tree_loading(fs_opts);
    let mut render = render::TreeRender::new(&mut t, rd_opts);

    match rm {
        args::RenderMethod::JustSummary => {
            println!("\n{}", render.tree.summary());
        }
        args::RenderMethod::NoInteractive => {
            print!("{}", render);
        }
        args::RenderMethod::FullInteractive => {
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
