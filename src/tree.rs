use std::cmp::max;
use std::collections::HashMap;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::{fmt, io};

use indextree::{Arena, NodeId};
use termion::color::{Bg, Fg, Reset};

use fs::{fs_to_tree, FsEntry};
use options::*;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

/****** Tree ******/

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
    n_files: usize,
    n_dirs: usize,
    pub(crate) render_opts: RenderOptions,
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.tree[self.root].data.name)?;

        for l in &self.lines.lines[1..] {
            writeln!(f, "{} {}", l.prefix, self.tree[l.node].data.name)?;
        }

        writeln!(f, "\n{}", self.summary())?;

        Ok(())
    }
}

impl Tree {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Tree::new_with_options(TreeOptions::new("."))
    }

    #[allow(dead_code)]
    pub fn new_from_dir<P: AsRef<Path>>(dir: &P) -> Self {
        let opt = TreeOptions::new(dir);
        Tree::new_with_options(opt)
    }

    pub fn new_with_options<P: AsRef<Path>>(options: TreeOptions<P>) -> Self {
        let (tree, root, n_files, n_dirs) = fs_to_tree(&options.root, &options.fs_opts);

        let lines = Tree::draw(&tree, root);

        Self {
            focused: if let Some(c) = tree[root].first_child() {
                c
            } else {
                root
            },
            tree,
            root,
            lines,
            n_files,
            n_dirs,
            render_opts: options.render_opts,
        }
    }

    #[allow(dead_code)]
    pub fn focused<'a>(&'a self) -> &'a FsEntry {
        &self.tree[self.focused].data
    }

    pub fn focus_up(&mut self) {
        self.focused = match self.tree[self.focused].parent() {
            None => self.focused,
            Some(p) => {
                if p == self.root {
                    self.focused
                } else {
                    p
                }
            }
        };
    }

    pub fn focus_down(&mut self) {
        self.focused = match self.tree[self.focused].first_child() {
            None => self.focused,
            Some(ps) => ps,
        };
    }

    pub fn focus_left(&mut self) {
        self.focused = match self.tree[self.focused].previous_sibling() {
            None => self.focused,
            Some(ps) => ps,
        };
    }

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
    pub fn render_around_focus<W: Write>(&self, writer: &mut W, n: i64) -> io::Result<()> {
        let y = self.lines.inds[&self.focused] as i64;
        let space = n / 2;

        let mut start = y - space;
        let mut end = y + space + n % 2;
        let count = self.lines.count as i64;

        if start < 0 {
            end += -1 * start;
            start = 0;
        }

        if end > count {
            start = max(0, start - (end - count));
            end = count;
        }

        self.render(writer, start as usize, end as usize, y as usize)
    }

    /// Render the lines of the tree in the range [top, bottom).
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        top: usize,
        bottom: usize,
        highlight: usize,
    ) -> io::Result<()> {
        print!("{}", Fg(self.render_opts.fg_color.deref()));
        for i in top..bottom {
            self.render_line(writer, i, i == highlight, i == bottom - 1)?;
        }
        print!("{}", Fg(Reset));

        Ok(())
    }

    /// Render a singe line of the tree.
    ///
    /// Uses \r\n as a line ending since when terminal is in raw mode \n
    /// alone does not move the cursor back to the beginning of the line.
    fn render_line<W: Write>(
        &self,
        writer: &mut W,
        ind: usize,
        highlight: bool,
        last: bool,
    ) -> io::Result<()> {
        let line = &self.lines.lines[ind];
        let ending = if last { "" } else { "\r\n" };

        if highlight {
            write!(
                writer,
                "{}{}{}{}{}{}",
                line.prefix,
                if line.prefix == "" { "" } else { " " },
                Bg(self.render_opts.bg_color.deref()),
                self.tree[line.node].data.name,
                Bg(Reset),
                ending,
            )
        } else {
            write!(
                writer,
                "{}{}{}{}",
                line.prefix,
                if line.prefix == "" { "" } else { " " },
                self.tree[line.node].data.name,
                if last { "" } else { "\r\n" }
            )
        }?;

        if last {
            writer.flush()
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Indent {
    Bar,
    Blank,
}

impl Tree {
    pub fn summary(&self) -> String {
        format!(
            "{} {}, {} {}",
            self.n_dirs,
            if self.n_dirs == 1 {
                "directory"
            } else {
                "directories"
            },
            self.n_files,
            if self.n_files == 1 { "file" } else { "files" }
        )
    }

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
        format!("{}", Tree::new_from_dir(dir))
    }

    #[test]
    fn test_draw_abs_path() {
        let dir = abs_test_dir("simple");

        let exp = format!(
            "{}\n{} {}\n{} {}\n\n{}\n",
            dir.display(),
            MID_BRANCH,
            "myfile",
            END_BRANCH,
            "myotherfile",
            "0 directories, 2 files",
        );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_rel_path() {
        let dir = test_dir("simple");

        let exp = format!(
            "{}\n{} {}\n{} {}\n\n{}\n",
            dir.display(),
            MID_BRANCH,
            "myfile",
            END_BRANCH,
            "myotherfile",
            "0 directories, 2 files",
        );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_dir() {
        let dir = test_dir("one_dir");

        let exp = format!(
            "{}\n{} {}\n{}{} {}\n{} {}\n\n{}\n",
            dir.display(),
            MID_BRANCH,
            "mydir",
            BAR_INDENT,
            END_BRANCH,
            "myfile",
            END_BRANCH,
            "myotherfile",
            "1 directory, 2 files",
        );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_link() {
        let dir = test_dir("link");

        let exp = format!(
            "{}\n{} {}\n{} {}\n\n{}\n",
            dir.display(),
            MID_BRANCH,
            "dest -> source",
            END_BRANCH,
            "source",
            "0 directories, 2 files",
        );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_focus() {
        let mut t = Tree::new_from_dir(&test_dir(""));
        assert_eq!("link", t.focused().name);
        t.focus_up();
        assert_eq!("link", t.focused().name);

        t.focus_right();
        assert_eq!("one_dir", t.focused().name);
        t.focus_down();
        assert_eq!("mydir", t.focused().name);

        t.focus_left();
        assert_eq!("mydir", t.focused().name);
        t.focus_right();
        assert_eq!("myotherfile", t.focused().name);

        t.focus_up();
        assert_eq!("one_dir", t.focused().name);
    }
}
