use std::io;

use termion;
use termion::clear::All;
use termion::cursor::{Goto, Hide, Show};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::{ToAlternateScreen, ToMainScreen};

use tree::Tree;

fn clear() {
    print!("{}", All);
    print!("{}", Goto(1, 1));
}

fn render_to_stdout(tree: &Tree) -> io::Result<()> {
    let mut stdout = io::stdout();

    clear();
    let (_, y) = termion::terminal_size()?;
    tree.render_around_focus(&mut stdout, y as i64)
}

pub fn navigate(tree: &mut Tree) {
    {
        // The following is necessary to properly read from stdin.
        // For details, see: https://github.com/ticki/termion/issues/42
        //
        // Wrapped in block so clenaup printing happens in non-raw mode.
        let _stdout = io::stdout().into_raw_mode().unwrap();

        println!("{}", ToAlternateScreen);
        println!("{}", Hide);

        render_to_stdout(tree)
            .map_err(|e| {
                println!("{}", Show);
                format!("Failed to render tree: {:?}", e)
            })
            .unwrap();

        let mut keys = io::stdin().keys();
        while let Some(Ok(key)) = keys.next() {
            match key {
                Key::Left => {
                    tree.focus_up();
                }
                Key::Right => {
                    tree.focus_down();
                }
                Key::Up => {
                    tree.focus_left();
                }
                Key::Down => {
                    tree.focus_right();
                }
                Key::Esc | Key::Char('q') | Key::Ctrl('c') => break,
                _ => {}
            }

            render_to_stdout(tree)
                .map_err(|e| {
                    println!("{}", Show);
                    format!("Failed to render tree: {:?}", e)
                })
                .unwrap();
        }
    }

    println!("{}", Show);
    println!("{}", ToMainScreen);

    println!("{}", tree.summary());
}
