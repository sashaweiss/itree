use std::cmp::{min, Ordering};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::{fmt, io};

use indextree::{Arena, NodeId};

use fs::{fs_to_tree, FsEntry};

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

#[derive(Debug)]
struct TreeLine {
    node: NodeId,
    prefix: String,
}

#[derive(Debug)]
struct TreeLines {
    inds: HashMap<NodeId, usize>,
    lines: Vec<TreeLine>,
    count: usize,
}

impl TreeLines {
    fn new() -> Self {
        TreeLines {
            inds: HashMap::new(),
            lines: Vec::new(),
            count: 0,
        }
    }

    fn add(&mut self, node: NodeId, prefix: String) {
        self.inds.insert(node, self.count);
        self.lines.push(TreeLine { node, prefix });
        self.count += 1;
    }
}

#[derive(Debug)]
pub struct Tree {
    tree: Arena<FsEntry>,
    root: NodeId,
    focused: NodeId,
    lines: TreeLines,
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for l in &self.lines.lines {
            writeln!(f, "{} {}", l.prefix, self.tree[l.node].data.name)?;
        }

        Ok(())
    }
}

impl Tree {
    pub fn new<P: AsRef<Path>>(dir: &P) -> Self {
        let (tree, root) = fs_to_tree(dir);

        let lines = Tree::draw(&tree, root);

        Self {
            tree,
            root,
            focused: root,
            lines,
        }
    }

    #[allow(dead_code)]
    pub fn focused<'a>(&'a self) -> &'a FsEntry {
        &self.tree[self.focused].data
    }

    #[allow(dead_code)]
    pub fn focus_up(&mut self) {
        self.focused = match self.tree[self.focused].parent() {
            None => self.focused,
            Some(p) => p,
        };
    }

    #[allow(dead_code)]
    pub fn focus_down(&mut self) {
        self.focused = match self.tree[self.focused].first_child() {
            None => self.focused,
            Some(ps) => ps,
        };
    }

    #[allow(dead_code)]
    pub fn focus_left(&mut self) {
        self.focused = match self.tree[self.focused].previous_sibling() {
            None => self.focused,
            Some(ps) => ps,
        };
    }

    #[allow(dead_code)]
    pub fn focus_right(&mut self) {
        self.focused = match self.tree[self.focused].next_sibling() {
            None => self.focused,
            Some(ps) => ps,
        };
    }
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
        let y = self.lines.inds[&self.focused];
        let space = n / 2;

        let (start, diff) = match space.cmp(&y) {
            Ordering::Less | Ordering::Equal => (y - space, 0),
            Ordering::Greater => (0, space - y),
        };

        let end = min(self.lines.count, y + space + diff + n % 2);

        self.render(writer, start, end)
    }

    /// Render the lines in the range [top, bottom).
    pub fn render<W: Write>(&self, writer: &mut W, top: usize, bottom: usize) -> io::Result<()> {
        for i in top..bottom {
            let line = &self.lines.lines[i];
            writeln!(writer, "{} {}", line.prefix, self.tree[line.node].data.name)?;
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
    fn draw(tree: &Arena<FsEntry>, root: NodeId) -> TreeLines {
        let mut tree_lines = TreeLines::new();

        // Draw the root
        tree_lines.add(root, String::new());

        // Draw the rest of the tree
        Tree::draw_from(&mut tree_lines, &tree, root, &mut vec![]);

        tree_lines
    }

    fn draw_from(
        tree_lines: &mut TreeLines,
        tree: &Arena<FsEntry>,
        root: NodeId,
        indents: &mut Vec<Indent>,
    ) {
        for child in root.children(&tree) {
            let last = Some(child) == tree[root].last_child();

            let mut prefix = String::new();
            for i in indents.iter() {
                prefix.push_str(match *i {
                    Indent::Bar => BAR_INDENT,
                    Indent::Blank => BLANK_INDENT,
                });
            }
            prefix.push_str(if last { END_BRANCH } else { MID_BRANCH });

            tree_lines.add(child, prefix);

            indents.push(if last { Indent::Blank } else { Indent::Bar });
            Tree::draw_from(tree_lines, tree, child, indents);
        }

        indents.pop();
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

    #[test]
    fn test_focus() {
        let mut t = Tree::new(&test_dir(""));
        assert_eq!("test", t.focused().name);
        t.focus_up();
        assert_eq!("test", t.focused().name);

        t.focus_down();
        assert_eq!("one_dir", t.focused().name);
        t.focus_left();
        assert_eq!("one_dir", t.focused().name);

        t.focus_right();
        assert_eq!("simple", t.focused().name);
        t.focus_right();
        assert_eq!("simple", t.focused().name);

        t.focus_up();
        assert_eq!("test", t.focused().name);
    }
}
