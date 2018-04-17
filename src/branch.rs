use super::cursor::{Cursor, new_cursor_bound_to_term};

use std::iter;
use std::path::Path;

use ignore::WalkBuilder;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

fn draw_branch(cur: &mut Cursor, entry: &Path, last: bool, indent: usize) {
    println!(
        "{}{} {}",
        iter::repeat("    ").take(indent).collect::<String>(),
        if last { END_BRANCH } else { MID_BRANCH },
        entry.display(),
        );
    cur.down();
}

fn draw_root(cur: &mut Cursor, entry: &Path) {
    println!("{}", entry.display());
    cur.down();
}

/// Draw the tree, starting with the given directory, starting at the top of the terminal space.
pub fn draw_from<P: AsRef<Path>>(dir: P) -> Cursor {
    let mut cur = new_cursor_bound_to_term();
    draw_from_with(&mut cur, dir.as_ref());
    cur
}

/// Draw the tree, starting with the given directory, from the given cursor.
fn draw_from_with(cur: &mut Cursor, dir: &Path) {
    let mut walk = WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .build()
        .peekable();

    draw_root(cur, dir);
    walk.next();

    while let Some(Ok(de)) = walk.next() {
        let last = match walk.peek() {
            Some(&Ok(ref next)) => next.depth() > de.depth(),
            Some(&Err(_)) => continue,
            None => true,
        };

        draw_branch(cur, &de.path(), last, de.depth() - 1);
    }
}
