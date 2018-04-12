extern crate termion;
extern crate tui;

mod cursor;
mod border;

use std::fs;
use std::io;
use std::cmp::Ordering;

fn main() {
    let _des = files_in_root();

    let mut cur = cursor::new_cursor_bound_to_term();

    border::draw_border(
        cur.bound_l,
        cur.bound_t,
        cur.bound_r - cur.bound_l,
        cur.bound_b - cur.bound_t,
    );

    cur.interact();
}

fn files_in_root() -> Vec<fs::DirEntry> {
    files_in_dir(&"./".to_string()).unwrap()
}

fn files_in_dir(dir: &str) -> io::Result<Vec<fs::DirEntry>> {
    let mut des: Vec<fs::DirEntry> = fs::read_dir(dir)?
        .filter(|de| de.is_ok())
        .map(|de| de.unwrap())
        .collect();

    des.sort_by(|f: &fs::DirEntry, s: &fs::DirEntry| -> Ordering {
        f.path().cmp(&s.path())
    });

    Ok(des)
}
