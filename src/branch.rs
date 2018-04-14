use super::cursor::{Cursor, new_cursor_bound_to_term};

use std::fs;
use std::io;
use std::fs::DirEntry;
use std::cmp::Ordering;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

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

fn draw_branch(cur: &mut Cursor, entry: &DirEntry, last: bool, prefix: &str) {
    if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
        println!(
            "{}{} {}",
            prefix,
            if last { END_BRANCH } else { MID_BRANCH },
            file_name
        );
        cur.down();
    }
}

/// Draw the tree, starting with the given directory, starting at the top of the terminal space.
pub fn draw_from(dir: &str) {
    draw_from_with(&mut new_cursor_bound_to_term(), dir);
}

/// Draw the tree, starting with the given directory, from the given cursor.
pub fn draw_from_with(cur: &mut Cursor, dir: &str) {
    let files = files_in_dir(dir).unwrap();

    let mut c = 0;
    for de in &files {
        c += 1;
        draw_branch(cur, de, c == files.len(), "");
    }

}
