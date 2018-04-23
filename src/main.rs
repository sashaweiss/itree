extern crate termion;
extern crate indextree;
extern crate ignore;

mod cursor;
mod draw;
mod fs;
mod navigate;

use std::io;

fn main() {
    let drawn_tree = draw::draw_rooted(&mut io::stdout(), &".");

    // The following is necessary to properly read from stdin.
    // For details, see: https://github.com/ticki/termion/issues/42
    // let _stdout = io::stdout().into_raw_mode().unwrap();
    // fs::navigate(io::stdin(), cursor::new_cursor_bound_to_term(), tree, root);
}
