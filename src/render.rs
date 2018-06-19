use std::fmt;
use std::io::{self, Write};
use std::ops::Deref;

use termion::color::{Bg, Fg, Reset};

use fs::FileType;
use options::RenderOptions;
use tree::Tree;

pub const FOLD_MARK: &str = "*";
pub const RESTRICTED_MARK: &str = " [error opening dir]";
pub const LINK_MARK: &str = " -> ";

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

pub struct TreeRender<'a> {
    tree: &'a mut Tree,
    opts: RenderOptions,
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

        let suffix = match &self.tree.tree[line.node].data.ft {
            FileType::File => String::new(),
            FileType::Dir => {
                if self.tree.lines.folded.contains(&ind) {
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
        };

        if focus {
            write!(
                writer,
                "{}{}{}{}{}{}{}",
                line.prefix,
                if line.prefix == "" { "" } else { " " },
                Bg(self.opts.bg_color.deref()),
                self.tree.tree[line.node].data.name,
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
                self.tree.tree[line.node].data.name,
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
