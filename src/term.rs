use std::io;

use termion;
use termion::clear::All;
use termion::cursor::Goto;

use tree::Tree;

pub fn render_to_stdout(tree: &Tree) -> io::Result<()> {
    let mut stdout = io::stdout();

    print!("{}", All);
    print!("{}", Goto(1, 1));

    let (_, y) = termion::terminal_size()?;
    tree.render(&mut stdout, y as usize)
}
