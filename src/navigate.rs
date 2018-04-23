use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use std::io;
use std::io::Read;

use indextree::{Arena, NodeId};

fn navigate<R>(read_from: R, cur: &mut cursor::Cursor, tree: &Arena<DirEntry>, mut curr: NodeId)
where
    R: Read,
{
    cur.draw();
    for c in read_from.keys() {
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
