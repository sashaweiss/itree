use std::cmp::max;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::{fmt, io};

use indextree::{Arena, NodeId};
use termion::color;

use fs::{fs_to_tree, FsEntry};

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

/****** TreeOptions ******/

#[derive(Debug)]
pub struct TreeOptions<P: AsRef<Path>> {
    pub(crate) root: P,
    pub(crate) max_depth: Option<usize>,
    pub(crate) follow_links: bool,
    pub(crate) max_filesize: Option<u64>,
    pub(crate) hidden: bool,
    pub(crate) no_ignore: bool,
    pub(crate) no_git_exclude: bool,
    pub(crate) custom_ignore: Vec<String>,
}

impl<P: AsRef<Path>> TreeOptions<P> {
    pub fn new(root: P) -> Self {
        Self {
            root,
            max_depth: None,
            follow_links: false,
            max_filesize: None,
            hidden: true,
            no_ignore: true,
            no_git_exclude: true,
            custom_ignore: Vec::new(),
        }
    }

    /// Set the root directory from which to build the tree.
    pub fn root(&mut self, root: P) -> &mut Self {
        self.root = root;
        self
    }

    /// Set a maximum depth for the tree to search. `None` indicates no limit.
    ///
    /// `None` by default.
    pub fn max_depth(&mut self, max_depth: Option<usize>) -> &mut Self {
        self.max_depth = max_depth;
        self
    }

    /// Set whether or not to follow links.
    ///
    /// Disabled by default.
    pub fn follow_links(&mut self, follow_links: bool) -> &mut Self {
        self.follow_links = follow_links;
        self
    }

    /// Set a maximum file size to include. `None` indicates no limit.
    ///
    /// `None` by default.
    pub fn max_filesize(&mut self, max_filesize: Option<u64>) -> &mut Self {
        self.max_filesize = max_filesize;
        self
    }

    /// Set whether or not to ignore hidden files.
    ///
    /// Enabled by default.
    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
    }

    /// Set whether or not to read `.[git]ignore` files.
    ///
    /// Enabled by default.
    pub fn no_ignore(&mut self, no_ignore: bool) -> &mut Self {
        self.no_ignore = no_ignore;
        self
    }

    /// Set whether or not to read `.git/info/exclude` files.
    ///
    /// Enabled by default.
    pub fn no_git_exclude(&mut self, no_git_exclude: bool) -> &mut Self {
        self.no_git_exclude = no_git_exclude;
        self
    }

    /// Add a custom ignore path.
    pub fn add_custom_ignore(&mut self, path: &str) -> &mut Self {
        self.custom_ignore.push(path.to_owned());
        self
    }
}

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
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.tree[self.root].data.name)?;

        for l in &self.lines.lines[1..] {
            writeln!(f, "{} {}", l.prefix, self.tree[l.node].data.name)?;
        }

        Ok(())
    }
}

impl Tree {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Tree::new_with_options(TreeOptions::new(".")).unwrap()
    }

    #[allow(dead_code)]
    fn new_from_dir<P: AsRef<Path>>(dir: &P) -> Self {
        let opt = TreeOptions::new(dir);
        Tree::new_with_options(opt).unwrap()
    }

    pub fn new_with_options<P: AsRef<Path>>(options: TreeOptions<P>) -> Result<Self, String> {
        let (tree, root) = fs_to_tree(options)?;

        let lines = Tree::draw(&tree, root);

        Ok(Self {
            focused: if let Some(c) = tree[root].first_child() {
                c
            } else {
                root
            },
            tree,
            root,
            lines,
        })
    }

    #[allow(dead_code)]
    pub fn focused<'a>(&'a self) -> &'a FsEntry {
        &self.tree[self.focused].data
    }

    #[allow(dead_code)]
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
    pub fn render_around_focus<W: Write>(&self, writer: &mut W, n: i64) -> io::Result<()> {
        let y = self.lines.inds[&self.focused] as i64;
        let space = n / 2;

        // The range is gonna be size <= n
        //
        // Want to start n/2 above y, or 0 if that's negative.
        // if its negative, want to add the underflow to the end.
        //
        // want to end n/2 below y, or lines.count if that's bigger.
        // if it's bigger, want to add the overflow to the start.
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
                color::Bg(color::Blue),
                self.tree[line.node].data.name,
                color::Bg(color::Reset),
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

    /// Render the lines of the tree in the range [top, bottom).
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        top: usize,
        bottom: usize,
        highlight: usize,
    ) -> io::Result<()> {
        for i in top..bottom {
            self.render_line(writer, i, i == highlight, i == bottom - 1)?;
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
        format!("{}", Tree::new_from_dir(dir))
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
        let mut t = Tree::new_from_dir(&test_dir(""));
        assert_eq!("resources/test", t.focused().name);
        t.focus_up();
        assert_eq!("resources/test", t.focused().name);

        t.focus_down();
        assert_eq!("one_dir", t.focused().name);
        t.focus_left();
        assert_eq!("one_dir", t.focused().name);

        t.focus_right();
        assert_eq!("simple", t.focused().name);
        t.focus_right();
        assert_eq!("simple", t.focused().name);

        t.focus_up();
        assert_eq!("resources/test", t.focused().name);
    }
}
