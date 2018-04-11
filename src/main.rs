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

use termion::input::TermRead;
use termion::cursor::DetectCursorPos;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Block, Borders, Widget};
use tui::layout::Rect;

fn main() {
    let mut terminal = Terminal::new(RawBackend::new().unwrap()).unwrap();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // terminal.clear().unwrap();

    let (_, cur_y) = stdout.cursor_pos().unwrap();
    Block::default().borders(Borders::ALL).render(
        &mut terminal,
        &Rect::new(
            0,
            cur_y - 1,
            10,
            5,
        ),
    );

    terminal.draw().unwrap();

    for _ in stdin.keys() {
        return;
    }
}
