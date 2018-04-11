use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Block, Borders, Widget};
use tui::layout::Rect;


pub fn draw_border(ogn_x: u16, ogn_y: u16, siz_x: u16, siz_y: u16) {
    let mut terminal = Terminal::new(RawBackend::new().unwrap()).unwrap();

    Block::default().borders(Borders::ALL).render(
        &mut terminal,
        &Rect::new(
            ogn_x - 1, // Termion and Tui aren't playing nice
            ogn_y,
            siz_x + 1, // Termion and Tui aren't playing nice
            siz_y,
        ),
    );

    terminal.draw().unwrap();
}
