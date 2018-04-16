extern crate termion;
extern crate ignore;

mod cursor;
mod branch;

use std::io;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    // cursor::hide();

    branch::draw_from(&"./");

    // cursor::show();
    // interact(cur);
}

#[allow(dead_code)]
fn interact(mut cur: cursor::Cursor) {
    let stdin = io::stdin();

    // The following is necessary to properly read from stdin.
    // For details, see: https://github.com/ticki/termion/issues/42
    let _stdout = io::stdout().into_raw_mode().unwrap();

    cur.draw();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') | Key::Ctrl('c') => break,
            Key::Left => {
                cur.left();
            }
            Key::Right => {
                cur.right();
            }
            Key::Up => {
                cur.up();
            }
            Key::Down => {
                cur.down();
            }
            _ => {}
        }
    }
}
