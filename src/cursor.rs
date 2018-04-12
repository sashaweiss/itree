use std::io;

use termion::terminal_size;
use termion::input::TermRead;
use termion::cursor::{DetectCursorPos, Goto};
use termion::event::Key;
use termion::raw::IntoRawMode;

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub bound_t: u16,
    pub bound_b: u16,
    pub bound_l: u16,
    pub bound_r: u16,
}

impl Cursor {
    pub fn interact(&mut self) {
        let stdin = io::stdin();

        self.draw();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Left => {
                    self.left();
                }
                Key::Right => {
                    self.right();
                }
                Key::Up => {
                    self.up();
                }
                Key::Down => {
                    self.down();
                }
                _ => {}
            }
        }
    }

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
        // Termion and Tui aren't playing nice
        if self.y < self.bound_b - 1 {
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

pub fn new_cursor_bound_to_term() -> Cursor {
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    let (_, _tl_y) = stdout.cursor_pos().unwrap();
    let (ogn_x, ogn_y) = (1, _tl_y - 1); // Termion and Tui aren't playing nice

    let (_br_x, _br_y) = terminal_size().unwrap();
    let (far_x, far_y) = (_br_x - ogn_x, _br_y - ogn_y);

    Cursor::new(ogn_x, ogn_y, ogn_y, ogn_y + far_y, ogn_x, ogn_x + far_x)
}
