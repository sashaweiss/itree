use std::io;

use termion::terminal_size;
use termion::raw::IntoRawMode;
use termion::cursor::{DetectCursorPos, Goto, Hide, Show};

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub bound_t: u16,
    pub bound_b: u16,
    pub bound_l: u16,
    pub bound_r: u16,
}

impl Cursor {
    pub fn draw(&self) {
        println!("{}", Goto(self.x, self.y));
    }

    #[allow(dead_code)]
    pub fn up(&mut self) {
        if self.y > self.bound_t {
            self.y -= 1;
        }
        self.draw();
    }

    #[allow(dead_code)]
    pub fn down(&mut self) {
        // Termion and Tui aren't playing nice
        if self.y < self.bound_b - 1 {
            self.y += 1;
        }
        self.draw();
    }

    #[allow(dead_code)]
    pub fn right(&mut self) {
        if self.x < self.bound_r {
            self.x += 1;
        }
        self.draw();
    }

    #[allow(dead_code)]
    pub fn left(&mut self) {
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

    Cursor {
        x: ogn_x,
        y: ogn_y,
        bound_t: ogn_y,
        bound_b: ogn_y + far_y,
        bound_l: ogn_x,
        bound_r: ogn_x + far_x,
    }
}

pub fn hide() {
    println!("{}", Hide);
}

pub fn show() {
    println!("{}", Show);
}
