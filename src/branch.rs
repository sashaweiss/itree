use std::iter;
use std::path::Path;
use std::io::Write;

use ignore::WalkBuilder;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

fn draw_branch<W: Write>(writer: &mut W, entry: &Path, last: bool, indent: usize) {
    writeln!(
        writer,
        "{}{} {}",
        iter::repeat("    ").take(indent).collect::<String>(),
        if last { END_BRANCH } else { MID_BRANCH },
        entry.display(),
        ).unwrap();
}

fn draw_root<W: Write>(writer: &mut W, entry: &Path) {
    writeln!(writer, "{}", entry.display()).unwrap();
}

/// Draw the tree, starting with the given directory.
pub fn draw_from<W: Write, P: AsRef<Path>>(writer: &mut W, dir: &P) {
    let mut walk = WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .build()
        .peekable();

    draw_root(writer, dir.as_ref());
    walk.next();

    while let Some(Ok(de)) = walk.next() {
        let last = match walk.peek() {
            Some(&Ok(ref next)) => next.depth() > de.depth(),
            Some(&Err(_)) => continue,
            None => true,
        };

        draw_branch(writer, &de.path(), last, de.depth() - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{Path, PathBuf};
    use std::env::temp_dir;
    use std::fs::{File, create_dir, create_dir_all, remove_dir_all};

    fn td() -> PathBuf {
        temp_dir().join("rusty-tree-tests")
    }

    fn setup() {
        let p = td();
        if !Path::new(&p).exists() {
            create_dir(p).unwrap();
        }
    }

    fn clean() {
        let p = td();
        if Path::new(&p).exists() {
            remove_dir_all(p).unwrap();
        }
    }

    fn create_files<P: AsRef<Path>>(names: &[P]) {
        for name in names {
            File::create(td().join(name)).unwrap();
        }
    }

    fn create_dirs<P: AsRef<Path>>(path_parts: &[P]) {
        let path = path_parts.iter().fold(PathBuf::new(), |acc, p| acc.join(p));
        create_dir_all(td().join(path)).unwrap();
    }

    #[test]
    fn test_draw_no_dir() {}

    #[test]
    fn test_draw_simple_dir() {}

    #[test]
    fn test_draw_nested_dir() {}
}
