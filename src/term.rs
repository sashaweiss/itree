use std::io;

use termion;
use termion::clear::All;
use termion::cursor::Goto;

use tree::Tree;

fn clear() {
    print!("{}", All);
    print!("{}", Goto(1, 1));
}

pub fn render_to_stdout(tree: &Tree) -> io::Result<()> {
    let mut stdout = io::stdout();

    clear();
    let (_, y) = termion::terminal_size()?;
    tree.render_around_focus(&mut stdout, y as usize)
}
