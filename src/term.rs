use std::io;

use termion;
use termion::clear::All;
use termion::cursor::{Goto, Hide, Show};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::{ToAlternateScreen, ToMainScreen};

use render::TreeRender;

fn clear() {
    print!("{}", All);
    print!("{}", Goto(1, 1));
}

fn render_to_stdout(render: &TreeRender) -> io::Result<()> {
    let mut stdout = io::stdout();

    clear();
    let (x, y) = termion::terminal_size()?;

    render.render_around_focus(&mut stdout, y as usize, x as usize)
}

pub fn navigate(render: &mut TreeRender) {
    {
        // The following is necessary to properly read from stdin.
        // For details, see: https://github.com/ticki/termion/issues/42
        //
        // Wrapped in block so clenaup printing happens in non-raw mode.
        let _stdout = io::stdout().into_raw_mode().unwrap();

        println!("{}", ToAlternateScreen);
        println!("{}", Hide);

        render_to_stdout(&render)
            .map_err(|e| {
                println!("{}", Show);
                format!("Failed to render tree: {:?}", e)
            })
            .unwrap();

        let mut keys = io::stdin().keys();
        while let Some(Ok(key)) = keys.next() {
            match key {
                Key::Left | Key::Char('h') => {
                    render.focus_up();
                }
                Key::Right | Key::Char('l') => {
                    render.focus_down();
                }
                Key::Up | Key::Char('k') => {
                    render.focus_left();
                }
                Key::Down | Key::Char('j') => {
                    render.focus_right();
                }
                Key::Char('f') => {
                    render.toggle_focus_fold();
                }
                Key::Esc | Key::Char('q') | Key::Ctrl('c') => break,
                _ => {}
            }

            render_to_stdout(&render)
                .map_err(|e| {
                    println!("{}", Show);
                    format!("Failed to render tree: {:?}", e)
                })
                .unwrap();
        }
    }

    println!("{}", Show);
    println!("{}", ToMainScreen);

    println!("{}", render.tree.summary());
}
