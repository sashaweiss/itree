use super::cursor::Cursor;

use std::fs::DirEntry;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

/// Draw a branch of the tree, and return how many vertical lines it took.
pub fn draw(cur: &mut Cursor, entry: &DirEntry, last: bool, prefix: &str) {
    if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
        println!(
            "{}{} {}",
            prefix,
            if last { END_BRANCH } else { MID_BRANCH },
            file_name
        );
        cur.down();

        if let Ok(true) = entry.file_type().and_then(|ft| Ok(ft.is_dir())) {
            println!("is a dir");
            cur.down();
        }
    }
}
