use std::fmt;
use std::io::{self, Write};
use std::ops::Deref;

use indextree::NodeId;
use termion::color::{Bg, Fg, Reset};

use fs::FileType;
use options::RenderOptions;
use tree::{PrefixPiece, Tree};

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";
pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

pub const FOLD_MARK: &str = "*";
pub const RESTRICTED_MARK: &str = " [error opening dir]";
pub const LINK_MARK: &str = " -> ";

pub struct TreeRender<'a> {
    pub tree: &'a mut Tree,
    opts: RenderOptions,
}

impl<'a> fmt::Display for TreeRender<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.tree.tree[self.tree.root].data.name)?;

        let mut l_ind = 1;
        while let Some(line) = &self.tree.lines.lines.get(l_ind) {
            writeln!(
                f,
                "{} {}{}",
                self.prefix_string(&line.prefix),
                self.tree.tree[line.node].data.name,
                self.suffix_for_node(line.node)
            )?;

            l_ind = line.next;
        }

        writeln!(f, "\n{}", self.tree.summary())?;

        Ok(())
    }
}

impl<'a> TreeRender<'a> {
    pub fn new(tree: &'a mut Tree, opts: RenderOptions) -> Self {
        Self { tree, opts }
    }

    pub fn focus_up(&mut self) {
        self.tree.focus_up();
    }

    pub fn focus_down(&mut self) {
        self.tree.focus_down();
    }

    pub fn focus_left(&mut self) {
        self.tree.focus_left();
    }

    pub fn focus_right(&mut self) {
        self.tree.focus_right();
    }

    pub fn toggle_focus_fold(&mut self) {
        self.tree.toggle_focus_fold();
    }

    fn prefix_string(&self, prefix: &Vec<PrefixPiece>) -> String {
        prefix.iter().fold(String::new(), |acc, pre| {
            acc + match pre {
                PrefixPiece::BarIndent => BAR_INDENT,
                PrefixPiece::BlankIndent => BLANK_INDENT,
                PrefixPiece::MidBranch => MID_BRANCH,
                PrefixPiece::EndBranch => END_BRANCH,
            }
        })
    }

    fn suffix_for_node(&self, node: NodeId) -> String {
        match &self.tree.tree[node].data.ft {
            FileType::File => String::new(),
            FileType::Dir => {
                if self.tree
                    .lines
                    .folded
                    .contains(&self.tree.lines.inds[&node])
                {
                    FOLD_MARK.to_owned()
                } else {
                    String::new()
                }
            }
            FileType::RestrictedDir => RESTRICTED_MARK.to_owned(),
            FileType::LinkTo(dest) => {
                let mut s = String::from(LINK_MARK);
                s.push_str(dest);
                s
            }
            FileType::Stdin => String::from("<stdin>"),
        }
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
        let y = self.tree.lines.inds[&self.tree.focused];
        let (mut start, end) = self.bounds_of_range_around_line(y, n, width);

        print!("{}", Fg(self.opts.fg_color.deref()));
        while start < end {
            let next = self.tree.lines.lines[start].next;
            let last = self.tree.lines.lines.get(next).is_none() || next >= end;

            self.render_line(writer, start, start == y, last)?;
            start = next
        }
        print!("{}", Fg(Reset));

        Ok(())
    }

    fn visual_lines_for_line(&self, l_ind: usize, width: usize) -> usize {
        let line = &self.tree.lines.lines[l_ind];
        let mut pl = line.prefix.len();
        if pl != 0 {
            pl += 1; // If not the root
        }
        pl += self.tree.tree[line.node].data.name.len();

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
            if let Some(prev) = self.tree.lines.lines[start].prev {
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
            let next = self.tree.lines.lines[end].next;
            if let Some(_) = self.tree.lines.lines.get(next) {
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
            if let Some(prev) = self.tree.lines.lines[start].prev {
                i += self.visual_lines_for_line(start, width);
                start = prev;
            } else {
                break;
            }
        }

        (start, end)
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
        let line = &self.tree.lines.lines[ind];
        let ending = if last { "" } else { "\r\n" };

        if focus {
            write!(
                writer,
                "{}{}{}{}{}{}{}",
                self.prefix_string(&line.prefix),
                if line.prefix.is_empty() { "" } else { " " },
                Bg(self.opts.bg_color.deref()),
                self.tree.tree[line.node].data.name,
                self.suffix_for_node(line.node),
                Bg(Reset),
                ending,
            )
        } else {
            write!(
                writer,
                "{}{}{}{}{}",
                self.prefix_string(&line.prefix),
                if line.prefix.is_empty() { "" } else { " " },
                self.tree.tree[line.node].data.name,
                self.suffix_for_node(line.node),
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
        let mut t = Tree::new_from_dir(dir);
        format!("{}", TreeRender::new(&mut t, RenderOptions::new()))
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
        let actual = format!("{}", TreeRender::new(&mut t, RenderOptions::new()));

        assert_eq!(exp, actual);

        t.focus_up();
        t.focus_right();
        t.toggle_focus_fold();

        let actual = format!("{}", TreeRender::new(&mut t, RenderOptions::new()));
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

        let actual = format!("{}", TreeRender::new(&mut t, RenderOptions::new()));
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

        let actual = format!("{}", TreeRender::new(&mut t, RenderOptions::new()));
        assert_eq!(exp_pre, actual);
    }
}
