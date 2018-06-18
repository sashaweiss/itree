use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::{fmt, io};

use indextree::{Arena, NodeId};
use termion::color::{Bg, Fg, Reset};

use fs::{fs_to_tree, FileType, FsEntry};
use options::*;

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

#[derive(Debug, Copy, Clone)]
enum Indent {
    Bar,
    Blank,
}
pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

pub const FOLD_MARK: &str = "*";
pub const RESTRICTED_MARK: &str = " [error opening dir]";
pub fn LINK_MARK(dest: &str) -> String {
    let mut s = String::from(" -> ");
    s.push_str(dest);

    s
}

#[derive(Debug)]
struct TreeLine {
    node: NodeId,
    prefix: String,
    prev: Option<usize>,
    next: usize,
}

#[derive(Debug)]
struct TreeLines {
    inds: HashMap<NodeId, usize>,
    lines: Vec<TreeLine>,
    folded: HashSet<usize>,
    count: usize,
}

impl TreeLines {
    fn new() -> Self {
        TreeLines {
            inds: HashMap::new(),
            lines: Vec::new(),
            folded: HashSet::new(),
            count: 0,
        }
    }

    fn add(&mut self, node: NodeId, prefix: String) {
        self.inds.insert(node, self.count);
        self.lines.push(TreeLine {
            node,
            prefix,
            prev: if self.count == 0 {
                None
            } else {
                Some(self.count - 1)
            },
            next: self.count + 1,
        });
        self.count += 1;
    }
}

#[derive(Debug)]
pub struct Tree {
    tree: Arena<FsEntry>,
    root: NodeId,
    focused: NodeId,
    focused_child: HashMap<NodeId, NodeId>,
    lines: TreeLines,
    n_files: usize,
    n_dirs: usize,
    pub(crate) render_opts: RenderOptions,
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.tree[self.root].data.name)?;

        let mut l_ind = 1;
        while let Some(line) = &self.lines.lines.get(l_ind) {
            let suffix = if self.lines.folded.contains(&l_ind) {
                FOLD_MARK
            } else if self.tree[line.node].data.ft == FileType::RestrictedDir {
                RESTRICTED_MARK
            } else {
                ""
            };

            writeln!(
                f,
                "{} {}{}",
                line.prefix, self.tree[line.node].data.name, suffix,
            )?;

            l_ind = line.next;
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
            focused_child: HashMap::new(),
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

    #[allow(dead_code)]
    fn line_for_node(&self, node: NodeId) -> &TreeLine {
        &self.lines.lines[self.lines.inds[&node]]
    }

    fn line_for_node_mut(&mut self, node: NodeId) -> &mut TreeLine {
        &mut self.lines.lines[self.lines.inds[&node]]
    }

    #[allow(dead_code)]
    fn focused_line(&self) -> &TreeLine {
        self.line_for_node(self.focused)
    }

    fn focused_line_mut(&mut self) -> &mut TreeLine {
        let f = self.focused;
        self.line_for_node_mut(f)
    }

    fn focused_line_ind(&self) -> usize {
        self.lines.inds[&self.focused]
    }

    pub fn focus_up(&mut self) {
        self.focused = match self.tree[self.focused].parent() {
            None => self.focused,
            Some(p) => {
                if p == self.root {
                    self.focused
                } else {
                    self.focused_child.insert(p, self.focused);
                    p
                }
            }
        };
    }

    pub fn focus_down(&mut self) {
        self.focused = match self.focused_child.get(&self.focused) {
            Some(&c) => c,
            None => match self.tree[self.focused].first_child() {
                None => self.focused,
                Some(ps) => {
                    if self.lines.folded.contains(&self.focused_line_ind()) {
                        self.focused
                    } else {
                        ps
                    }
                }
            },
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

    pub fn toggle_focus_fold(&mut self) {
        if self.tree[self.focused].data.ft == FileType::Dir {
            if self.lines.folded.contains(&self.focused_line_ind()) {
                self.unfold_focus();
            } else {
                self.fold_focus();
            }
        }
    }

    fn unfold_focus(&mut self) {
        let f_ind = self.focused_line_ind();

        let mut ptr = self.focused;
        while let Some(c) = self.tree[ptr].last_child() {
            ptr = c;
        }

        // If the focus's next is in the tree,
        // set its previous to the new previous
        let n_ind = self.lines.lines[f_ind].next;
        if n_ind < self.lines.count {
            self.lines.lines[n_ind].prev = Some(self.lines.inds[&ptr]);
        }

        // Mark this line as unfolded
        self.lines.folded.remove(&f_ind);

        // Set the focus's next to the focus + 1
        let fl = self.focused_line_mut();
        fl.next = f_ind + 1;
    }

    fn fold_focus(&mut self) {
        if !self.tree[self.focused]
            .data
            .de
            .file_type()
            .unwrap()
            .is_dir()
        {
            return;
        }

        let mut ptr = Some(self.focused);
        while let Some(p) = ptr {
            if let Some(n) = self.tree[p].next_sibling() {
                ptr = Some(n);
                break;
            } else {
                ptr = self.tree[p].parent();
            }
        }

        let new_next = match ptr {
            Some(nn) => self.lines.inds[&nn],
            None => self.lines.count,
        };

        // If the focus's new_next is in the tree,
        // set its previous to the focus
        if new_next < self.lines.count {
            self.lines.lines[new_next].prev = Some(self.lines.inds[&self.focused]);
        }

        // Mark this line folded
        let f_ind = self.focused_line_ind();
        self.lines.folded.insert(f_ind);

        // Set the focus's next to the new_next
        let fl = self.focused_line_mut();
        fl.next = new_next;
    }

    /****** Precomputing drawn lines ******/

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

    /****** Rendering and paging ******/

    fn visual_lines_for_line(&self, l_ind: usize, width: usize) -> usize {
        let line = &self.lines.lines[l_ind];
        let mut pl = line.prefix.len();
        if pl != 0 {
            pl += 1; // If not the root
        }
        pl += self.tree[line.node].data.name.len();

        pl / width + 1
    }

    /// Find the bounds of the range of n consecutively renderable lines
    /// around a given line.
    ///
    /// Lines are considered consecutive if they follow each other in the
    /// doubly-linked list in which a line's `next` and `prev` fields comprise
    /// the edges.
    ///
    /// The range will include n/2 lines above and n/2 lines below the given line.
    /// If the given line is within n/2 lines of the top or bottom of the tree,
    /// the remaining space will be used on the other side.
    fn bounds_of_range_around_line(&self, line: usize, n: usize, width: usize) -> (usize, usize) {
        let space = n / 2;

        // Roll the start back n/2 spaces. If fewer, save the diff.
        let mut start = line;
        let mut start_diff = 0;
        let mut i = 0;
        while i < space {
            if let Some(prev) = self.lines.lines[start].prev {
                i += self.visual_lines_for_line(start, width);
                start = prev;
            } else {
                start_diff = space - i;
                break;
            }
        }

        // Roll the end forward n/2 + start_diff spaces. If fewer, save the diff.
        let mut end = line;
        let mut end_diff = 0;
        let end_max = space + n % 2 + start_diff;
        let mut i = 0;
        while i < end_max {
            let next = self.lines.lines[end].next;
            if let Some(_) = self.lines.lines.get(next) {
                i += self.visual_lines_for_line(end, width);
                end = next;
            } else {
                end += 1;
                end_diff = end_max - i - 1;
                break;
            }
        }

        // Roll the start back at most an additional end_diff spaces.
        let mut i = 0;
        while i < end_diff {
            if let Some(prev) = self.lines.lines[start].prev {
                i += self.visual_lines_for_line(start, width);
                start = prev;
            } else {
                break;
            }
        }

        (start, end)
    }

    /// Render at most n consecutive lines of the tree around the focused node.
    ///
    /// Lines are considered consecutive if they are adjacent in the
    /// doubly-linked list of lines in which a line's `next` and `prev`
    /// fields comprise the links.
    pub fn render_around_focus<W: Write>(
        &self,
        writer: &mut W,
        n: usize,
        width: usize,
    ) -> io::Result<()> {
        let y = self.lines.inds[&self.focused];
        let (mut start, end) = self.bounds_of_range_around_line(y, n, width);

        print!("{}", Fg(self.render_opts.fg_color.deref()));
        while start < end {
            let next = self.lines.lines[start].next;
            let last = self.lines.lines.get(next).is_none() || next >= end;

            self.render_line(writer, start, start == y, last)?;
            start = next
        }
        print!("{}", Fg(Reset));

        Ok(())
    }

    /// Render a single line of the tree.
    ///
    /// Uses \r\n as a line ending since when terminal is in raw mode \n
    /// alone does not move the cursor back to the beginning of the line.
    fn render_line<W: Write>(
        &self,
        writer: &mut W,
        ind: usize,
        focus: bool,
        last: bool,
    ) -> io::Result<()> {
        let line = &self.lines.lines[ind];
        let ending = if last { "" } else { "\r\n" };

        let suffix = match &self.tree[line.node].data.ft {
            FileType::File => String::new(),
            FileType::Dir => {
                if self.lines.folded.contains(&ind) {
                    FOLD_MARK.to_owned()
                } else {
                    String::new()
                }
            }
            FileType::RestrictedDir => RESTRICTED_MARK.to_owned(),
            FileType::LinkTo(dest) => LINK_MARK(&dest),
        };

        if focus {
            write!(
                writer,
                "{}{}{}{}{}{}{}",
                line.prefix,
                if line.prefix == "" { "" } else { " " },
                Bg(self.render_opts.bg_color.deref()),
                self.tree[line.node].data.name,
                suffix,
                Bg(Reset),
                ending,
            )
        } else {
            write!(
                writer,
                "{}{}{}{}{}",
                line.prefix,
                if line.prefix == "" { "" } else { " " },
                self.tree[line.node].data.name,
                suffix,
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

    #[test]
    fn test_fold() {
        let mut t = Tree::new_from_dir(&test_dir(""));
        t.focus_right();
        t.focus_down();
        t.toggle_focus_fold();

        let exp = format!(
            "{}\n{} {}\n{}{} {}\n{}{} {}\n{} {}\n{}{} {}\n{}{} {}\n{} {}\n{}{} {}\n{}{} {}\n\n{}\n",
            "resources/test",
            MID_BRANCH,
            "link",
            BAR_INDENT,
            MID_BRANCH,
            "dest -> source",
            BAR_INDENT,
            END_BRANCH,
            "source",
            MID_BRANCH,
            "one_dir",
            BAR_INDENT,
            MID_BRANCH,
            "mydir*",
            BAR_INDENT,
            END_BRANCH,
            "myotherfile",
            END_BRANCH,
            "simple",
            BLANK_INDENT,
            MID_BRANCH,
            "myfile",
            BLANK_INDENT,
            END_BRANCH,
            "myotherfile",
            "4 directories, 6 files",
        );
        let actual = format!("{}", t);

        assert_eq!(exp, actual);

        t.focus_up();
        t.focus_right();
        t.toggle_focus_fold();

        let actual = format!("{}", t);
        let exp_pre = format!(
            "{}\n{} {}\n{}{} {}\n{}{} {}\n{} {}\n{}{} {}\n{}{} {}\n{} {}\n\n{}\n",
            "resources/test",
            MID_BRANCH,
            "link",
            BAR_INDENT,
            MID_BRANCH,
            "dest -> source",
            BAR_INDENT,
            END_BRANCH,
            "source",
            MID_BRANCH,
            "one_dir",
            BAR_INDENT,
            MID_BRANCH,
            "mydir*",
            BAR_INDENT,
            END_BRANCH,
            "myotherfile",
            END_BRANCH,
            "simple*",
            "4 directories, 6 files",
        );

        assert_eq!(exp_pre, actual);

        t.focus_left();
        t.toggle_focus_fold();

        let actual = format!("{}", t);
        let exp = format!(
            "{}\n{} {}\n{}{} {}\n{}{} {}\n{} {}\n{} {}\n\n{}\n",
            "resources/test",
            MID_BRANCH,
            "link",
            BAR_INDENT,
            MID_BRANCH,
            "dest -> source",
            BAR_INDENT,
            END_BRANCH,
            "source",
            MID_BRANCH,
            "one_dir*",
            END_BRANCH,
            "simple*",
            "4 directories, 6 files",
        );

        assert_eq!(exp, actual);

        t.toggle_focus_fold();

        let actual = format!("{}", t);
        assert_eq!(exp_pre, actual);
    }
}
