use std::cmp::{min, Ordering};
use std::io::Write;
use std::path::Path;
use std::{fmt, io};

use ignore::DirEntry;
use indextree::{Arena, NodeId};

use fs::fs_to_tree;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

#[derive(Debug)]
pub struct TreeEntry {
    pub de: DirEntry,
    pub loc: (usize, usize),
    pub name: String,
}

#[derive(Debug)]
pub struct Tree {
    tree: Arena<TreeEntry>,
    root: NodeId,
    focused: NodeId,
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
        let (tree, root) = fs_to_tree(dir);

        let mut v = Vec::new();
        Tree::draw(&mut v, &tree, root);

        let lines = String::from_utf8_lossy(&v)
            .lines()
            .map(|e| e.to_owned())
            .collect::<Vec<String>>();

        Self {
            tree,
            root,
            focused: root,
            lines,
        }
    }

    // pub fn focused<'a>(&'a self) -> &'a TreeEntry {
    //     &self.tree[self.focused].data
    // }
}

impl Tree {
    /// Render n lines of the tree, around the focused node.
    ///
    /// n/2 lines above the node and n/2 lines below will be rendered.
    /// If the focus is within n/2 lines of the top or bottom of the tree,
    /// the remaining space will be used on the other side.
    ///
    /// TODO: handle line wrappings
    pub fn render_around_focus<W: Write>(&self, writer: &mut W, n: usize) -> io::Result<()> {
        let (_, y) = self.tree[self.focused].data.loc;
        let space = n / 2;

        let (start, diff) = match space.cmp(&y) {
            Ordering::Less | Ordering::Equal => (y - space, 0),
            Ordering::Greater => (0, space - y),
        };

        let end = min(self.lines.len(), y + space + diff + n % 2);

        self.render(writer, start, end)
    }

    /// Render the lines in the range [top, bottom).
    pub fn render<W: Write>(&self, writer: &mut W, top: usize, bottom: usize) -> io::Result<()> {
        for i in top..bottom {
            writeln!(writer, "{}", self.lines[i])?;
        }

        Ok(())
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
            let te = &tree[child].data;
            let last = Some(child) == tree[root].last_child();

            let mut idt = String::new();
            for i in indents.iter() {
                idt.push_str(match *i {
                    Indent::Bar => BAR_INDENT,
                    Indent::Blank => BLANK_INDENT,
                });
            }

            Tree::draw_branch(writer, &te.name, last, &idt);

            indents.push(if last { Indent::Blank } else { Indent::Bar });
            Tree::draw_from(writer, tree, child, indents);
        }

        indents.pop();
    }

    fn draw_branch<W: Write>(writer: &mut W, name: &str, last: bool, prefix: &str) {
        writeln!(
            writer,
            "{}{} {}",
            prefix,
            if last { END_BRANCH } else { MID_BRANCH },
            name,
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
