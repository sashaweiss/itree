use std::path::Path;
use std::io::Write;

use indextree::{Arena, NodeId};
use ignore::DirEntry;

use fs::collect_fs;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

#[derive(Copy, Clone)]
enum Indents {
    Bar,
    Blank,
}

/// Draw the tree, starting with the given directory.
pub fn draw_rooted<W: Write, P: AsRef<Path>>(writer: &mut W, dir: &P) {
    let (tree, root) = collect_fs(dir);

    draw_root(writer, dir.as_ref());
    draw_tree(writer, &tree, root, &mut vec![]);
}

fn draw_tree<W: Write>(
    writer: &mut W,
    tree: &Arena<DirEntry>,
    root: NodeId,
    indents: &mut Vec<Indents>,
) {
    for child in root.children(&tree) {
        let de = &tree[child].data;
        let last = Some(child) == tree[root].last_child();

        let mut idt = String::new();
        indents.iter().for_each(|i| {
            idt.push_str(match *i {
                Indents::Bar => BAR_INDENT,
                Indents::Blank => BLANK_INDENT,
            })
        });
        draw_branch(writer, de.path(), last, &idt);

        indents.push(if last { Indents::Blank } else { Indents::Bar });
        draw_tree(writer, tree, child, indents);
    }

    indents.pop();
}

fn draw_branch<W: Write>(writer: &mut W, entry: &Path, last: bool, prefix: &str) {
    let file_name = match entry.file_name() {
        Some(name) => name,
        None => return,
    }.to_string_lossy();

    writeln!(
        writer,
        "{}{} {}",
        prefix,
        if last { END_BRANCH } else { MID_BRANCH },
        file_name,
        ).unwrap();
}

fn draw_root<W: Write>(writer: &mut W, entry: &Path) {
    writeln!(writer, "{}", entry.display()).unwrap();
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
                "myfile",
                END_BRANCH,
                "myotherfile",
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
                "myfile",
                END_BRANCH,
                "myotherfile",
            );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_dir() {
        let dir = test_dir("one_dir");

        let exp =
            format!(
            "{}\n{} {}\n{}{} {}\n{} {}\n",
            dir.display(),
            MID_BRANCH,
            "mydir",
            BAR_INDENT,
            END_BRANCH,
            "myfile",
            END_BRANCH,
            "myotherfile",
         );

        assert_eq!(exp, draw_to_string(&dir));
    }
}
