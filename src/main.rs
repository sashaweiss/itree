// extern crate termion;

// use std::fs;
// use std::cmp::Ordering;

// fn main() {
//     let mut des = files_in_dir(&"./".to_string());

//     des.sort_by(|f: &fs::DirEntry, s: &fs::DirEntry| -> Ordering {
//         f.path().cmp(&s.path())
//     });

//     println!(".");
//     for de in des {
//         println!("|__ {}", de.path().display());
//     }
// }

// fn files_in_dir(dir: &str) -> Vec<fs::DirEntry> {
//     fs::read_dir(dir)
//         .unwrap()
//         .filter(|de| de.is_ok())
//         .map(|de| de.unwrap())
//         .collect()
// }

extern crate termion;
extern crate tui;

use std::io;

use termion::terminal_size;
use termion::input::TermRead;
use termion::cursor::{DetectCursorPos, Goto};
use termion::event::Key;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Block, Borders, Widget};
use tui::layout::Rect;

struct Cursor {
    x: u16,
    y: u16,
    bound_t: u16,
    bound_b: u16,
    bound_l: u16,
    bound_r: u16,
}

impl Cursor {
    fn new(x: u16, y: u16, bound_t: u16, bound_b: u16, bound_l: u16, bound_r: u16) -> Cursor {
        Cursor {
            x: x,
            y: y,
            bound_t: bound_t,
            bound_b: bound_b,
            bound_l: bound_l,
            bound_r: bound_r,
        }
    }

    fn draw(&self) {
        println!("{}", Goto(self.x, self.y));
    }

    fn up(&mut self) {
        if self.y > self.bound_t {
            self.y -= 1;
        }
        self.draw();
    }

    fn down(&mut self) {
        if self.y < self.bound_b {
            self.y += 1;
        }
        self.draw();
    }

    fn right(&mut self) {
        if self.x < self.bound_r {
            self.x += 1;
        }
        self.draw();
    }

    fn left(&mut self) {
        if self.x > self.bound_l {
            self.x -= 1;
        }
        self.draw();
    }
}

fn main() {
    let mut terminal = Terminal::new(RawBackend::new().unwrap()).unwrap();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let (_, _tl_y) = stdout.cursor_pos().unwrap();
    let (tl_x, tl_y) = (1, _tl_y - 1);
    let (br_x, br_y) = terminal_size().unwrap();

    Block::default().borders(Borders::ALL).render(
        &mut terminal,
        &Rect::new(
            tl_x - 1, // tui and termion disagree on indexing
            tl_y,
            10,
            5,
        ),
    );

    terminal.draw().unwrap();

    let mut cur = Cursor::new(tl_x, tl_y, tl_y, br_y, tl_x, br_x);

    cur.draw();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
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
