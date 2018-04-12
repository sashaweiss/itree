extern crate termion;
extern crate tui;

mod cursor;
mod border;

use std::fs;
use std::cmp::Ordering;

fn main() {
    let _des = files_in_dir(&"./".to_string());

    let mut cur = cursor::new_cursor_bound_to_term();

    border::draw_border(
        cur.bound_l,
        cur.bound_t,
        cur.bound_r - cur.bound_l,
        cur.bound_b - cur.bound_t,
    );

    cur.interact();
}

fn files_in_dir(dir: &str) -> Vec<fs::DirEntry> {
    let mut des: Vec<fs::DirEntry> = fs::read_dir(dir)
        .unwrap()
        .filter(|de| de.is_ok())
        .map(|de| de.unwrap())
        .collect();

    des.sort_by(|f: &fs::DirEntry, s: &fs::DirEntry| -> Ordering {
        f.path().cmp(&s.path())
    });

    des
}
