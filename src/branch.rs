use super::cursor::{Cursor, new_cursor_bound_to_term};

use std::fs;
use std::io;
use std::iter;
use std::cmp::Ordering;
use std::path::Path;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

fn files_in_dir(dir: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut des: Vec<fs::DirEntry> = fs::read_dir(dir)?
        .filter(|de| de.is_ok())
        .map(|de| de.unwrap())
        .collect();

    des.sort_by(|f: &fs::DirEntry, s: &fs::DirEntry| -> Ordering {
        f.path().cmp(&s.path())
    });

    Ok(des)
}

fn draw_branch(cur: &mut Cursor, entry: &Path, last: bool, indent: usize) {
    println!(
        "{}{} {}",
        iter::repeat("\t").take(indent).collect::<String>(),
        if last { END_BRANCH } else { MID_BRANCH },
        entry.display(),
        );
    cur.down();
}

/// Draw the tree, starting with the given directory, starting at the top of the terminal space.
pub fn draw_from<P: AsRef<Path>>(dir: P) {
    draw_from_with(&mut new_cursor_bound_to_term(), dir.as_ref(), 0)
}

/// Draw the tree, starting with the given directory, from the given cursor.
fn draw_from_with(cur: &mut Cursor, dir: &Path, indent: usize) {
    let files = files_in_dir(dir).unwrap();

    let mut c = 0;
    for de in &files {
        c += 1;

        draw_branch(cur, &de.path(), c == files.len(), indent);

        if let Ok(true) = de.file_type().and_then(|ft| Ok(ft.is_dir())) {
            draw_from_with(cur, &dir.join(de.file_name()), indent + 1);
        }
    }
}
