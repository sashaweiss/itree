use std::cmp::min;
use std::io::Write;
use std::path::Path;
use std::{fmt, io};

use indextree::{Arena, NodeId};

use termion;
use termion::clear::All;
use termion::cursor::Goto;

use fs::{collect_fs, TreeEntry};

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

#[derive(Debug)]
pub struct Tree {
    tree: Arena<TreeEntry>,
    root: NodeId,
    lines: Vec<String>,
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for l in &self.lines {
            writeln!(f, "{}", l)?;
        }

        Ok(())
    }
}

impl Tree {
    pub fn new<P: AsRef<Path>>(dir: &P) -> Self {
        let (tree, root) = collect_fs(dir);

        let mut v = Vec::new();
        Tree::draw(&mut v, &tree, root);

        let lines = String::from_utf8_lossy(&v)
            .lines()
            .map(|e| e.to_owned())
            .collect::<Vec<String>>();

        Self { tree, root, lines }
    }
}

impl Tree {
    /// Render lines of the tree in the interval [top, bottom).
    fn render<W: Write>(&self, writer: &mut W, top: usize, bottom: usize) -> io::Result<()> {
        for i in top..min(self.lines.len(), bottom) {
            writeln!(writer, "{}", self.lines[i])?;
        }

        Ok(())
    }

    pub fn render_to_term(&self, start: usize) -> io::Result<()> {
        let mut stdout = io::stdout();

        print!("{}", All);
        print!("{}", Goto(1, 1));

        let (_, y) = termion::terminal_size()?;
        self.render(&mut stdout, start, start + y as usize)
    }
}

#[derive(Debug, Copy, Clone)]
enum Indent {
    Bar,
    Blank,
}

impl Tree {
    fn draw<W: Write>(writer: &mut W, tree: &Arena<TreeEntry>, root: NodeId) {
        // Draw the root
        writeln!(writer, "{}", tree[root].data.de.path().display()).unwrap();

        // Draw the rest of the tree
        Tree::draw_from(writer, &tree, root, &mut vec![]);
    }

    fn draw_from<W: Write>(
        writer: &mut W,
        tree: &Arena<TreeEntry>,
        root: NodeId,
        indents: &mut Vec<Indent>,
    ) {
        for child in root.children(&tree) {
            let de = &tree[child].data.de;
            let last = Some(child) == tree[root].last_child();

            let mut idt = String::new();
            for i in indents.iter() {
                idt.push_str(match *i {
                    Indent::Bar => BAR_INDENT,
                    Indent::Blank => BLANK_INDENT,
                });
            }

            Tree::draw_branch(writer, de.path(), last, &idt);

            indents.push(if last { Indent::Blank } else { Indent::Bar });
            Tree::draw_from(writer, tree, child, indents);
        }

        indents.pop();
    }

    fn draw_branch<W: Write>(writer: &mut W, entry: &Path, last: bool, prefix: &str) {
        let file_name = match entry.file_name() {
            Some(name) => name.to_str().unwrap_or("<node name non-UTF8"),
            None => "<node name unknown>",
        };

        writeln!(
            writer,
            "{}{} {}",
            prefix,
            if last { END_BRANCH } else { MID_BRANCH },
            file_name,
        ).unwrap();
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
        format!("{}", Tree::new(dir))
    }

    #[test]
    fn test_draw_abs_path() {
        let dir = abs_test_dir("simple");

        let exp = format!(
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

        let exp = format!(
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

        let exp = format!(
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
