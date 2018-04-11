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
use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{Block, Borders, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};

fn main() {
    let mut terminal = Terminal::new(MouseBackend::new().unwrap()).unwrap();
    let stdin = io::stdin();
    terminal.hide_cursor().unwrap();

    let mut term_size = terminal.size().unwrap();
    draw(&mut terminal, &term_size);
    for c in stdin.keys() {
        let size = terminal.size().unwrap();
        if term_size != size {
            terminal.resize(size).unwrap();
            term_size = size;
        }
        draw(&mut terminal, &term_size);
        let evt = c.unwrap();
        if evt == event::Key::Char('q') {
            break;
        }
    }
    terminal.show_cursor().unwrap();
}

fn draw(t: &mut Terminal<MouseBackend>, size: &Rect) {
    Block::default().borders(Borders::ALL).render(
        t,
        &Rect::new(
            0,
            0,
            10,
            5,
        ),
    );

    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    // Block::default().borders(Borders::ALL).render(t, size);
    // Group::default()
    //     .direction(Direction::Vertical)
    //     .margin(4)
    //     .sizes(&[Size::Percent(50), Size::Percent(50)])
    //     .render(t, size, |t, chunks| {
    //         Group::default()
    //             .direction(Direction::Horizontal)
    //             .sizes(&[Size::Percent(50), Size::Percent(50)])
    //             .render(t, &chunks[0], |t, chunks| {
    //                 Block::default()
    //                     .title("With background")
    //                     .title_style(Style::default().fg(Color::Yellow))
    //                     .style(Style::default().bg(Color::Green))
    //                     .render(t, &chunks[0]);
    //                 Block::default()
    //                     .title("Styled title")
    //                     .title_style(Style::default().fg(Color::White).bg(Color::Red).modifier(
    //                         Modifier::Bold,
    //                     ))
    //                     .render(t, &chunks[1]);
    //             });
    //         Group::default()
    //             .direction(Direction::Horizontal)
    //             .sizes(&[Size::Percent(50), Size::Percent(50)])
    //             .render(t, &chunks[1], |t, chunks| {
    //                 Block::default()
    //                     .title("With borders")
    //                     .borders(Borders::ALL)
    //                     .render(t, &chunks[0]);
    //                 Block::default()
    //                     .title("With styled borders")
    //                     .border_style(Style::default().fg(Color::Cyan))
    //                     .borders(Borders::LEFT | Borders::RIGHT)
    //                     .render(t, &chunks[1]);
    //             });
    //     });

    t.draw().unwrap();
}
