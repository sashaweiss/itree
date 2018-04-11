extern crate termion;

use termion::{color, cursor, style};

use std::io;
use std::io::Write;

fn main() {
    print!("{}", cursor::Show);
    // println!("{}Red", color::Fg(color::Red));
    // println!("{}Blue", color::Fg(color::Blue));
    // println!("{}Blue'n'Bold{}", style::Bold, style::Reset);
    // println!("{}Just plain italic", style::Italic);

    print!(
        "{}{}Stuff",
        termion::clear::All,
        termion::cursor::Goto(10, 10)
    );

    io::stdout().flush().unwrap();

    // loop {}
}
