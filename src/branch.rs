use std::iter;
use std::path::Path;
use std::io::Write;

use ignore::WalkBuilder;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";
pub const INDENT: &str = "    ";

fn draw_branch<W: Write>(writer: &mut W, entry: &Path, last: bool, indent: usize) {
    let file_name = match entry.file_name() {
        Some(name) => name,
        None => return,
    }.to_string_lossy();

    writeln!(
        writer,
        "{}{} {}",
        iter::repeat(INDENT).take(indent).collect::<String>(),
        if last { END_BRANCH } else { MID_BRANCH },
        file_name,
        ).unwrap();
}

fn draw_root<W: Write>(writer: &mut W, entry: &Path) {
    writeln!(writer, "{}", entry.display()).unwrap();
}

/// Draw the tree, starting with the given directory.
pub fn draw_rooted<W: Write, P: AsRef<Path>>(writer: &mut W, dir: &P) {
    let mut walk = WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .build()
        .peekable();

    draw_root(writer, dir.as_ref());
    walk.next();

    while let Some(Ok(de)) = walk.next() {
        let last = match walk.peek() {
            Some(&Ok(ref next)) => next.depth() != de.depth(),
            Some(&Err(_)) => continue,
            None => true,
        };

        draw_branch(writer, &de.path(), last, de.depth() - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    fn test_dir(dir: &str) -> PathBuf {
        PathBuf::new().join("resources/test").join(dir)
    }

    fn abs_test_dir(dir: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(test_dir(dir))
    }

    fn draw_to_string(dir: &PathBuf) -> String {
        let mut actual = Vec::new();
        draw_rooted(&mut actual, dir);

        String::from_utf8(actual).unwrap()
    }

    #[test]
    fn test_draw_abs_path() {
        let dir = abs_test_dir("simple");

        let exp =
            format!(
                "{}\n{} {}\n{} {}\n",
                dir.display(),
                MID_BRANCH,
                "myotherfile",
                END_BRANCH,
                "myfile",
            );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_rel_path() {
        let dir = test_dir("simple");

        let exp =
            format!(
                "{}\n{} {}\n{} {}\n",
                dir.display(),
                MID_BRANCH,
                "myotherfile",
                END_BRANCH,
                "myfile",
            );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_dir() {
        let dir = test_dir("one_dir");

        let exp =
            format!(
            "{}\n{} {}\n{} {}\n{}{} {}\n",
            dir.display(),
            MID_BRANCH,
            "myotherfile",
            END_BRANCH,
            "mydir",
            INDENT,
            END_BRANCH,
            "myfile",
         );

        assert_eq!(exp, draw_to_string(&dir));
    }
}
