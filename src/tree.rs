use std::io::Write;
use std::path::Path;

use indextree::{Arena, NodeId};

use fs::{collect_fs, TreeEntry};

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";

pub const BLANK_INDENT: &str = "    ";
pub const BAR_INDENT: &str = "│   ";

#[derive(Debug, Copy, Clone)]
enum Indent {
    Bar,
    Blank,
}

#[derive(Debug)]
pub struct Tree {
    tree: Arena<TreeEntry>,
    root: NodeId,
}

impl Tree {
    pub fn new<P: AsRef<Path>>(dir: &P) -> Self {
        let (tree, root) = collect_fs(dir);

        Self { tree, root }
    }

    pub fn draw<W: Write>(&self, writer: &mut W) {
        // Draw the root
        writeln!(writer, "{}", self.tree[self.root].data.de.path().display()).unwrap();

        // Draw the rest of the tree
        self.draw_from(writer, self.root, &mut vec![]);
    }

    fn draw_from<W: Write>(&self, writer: &mut W, root: NodeId, indents: &mut Vec<Indent>) {
        for child in root.children(&self.tree) {
            let de = &self.tree[child].data.de;
            let last = Some(child) == self.tree[root].last_child();

            let mut idt = String::new();
            for i in indents.iter() {
                idt.push_str(match *i {
                    Indent::Bar => BAR_INDENT,
                    Indent::Blank => BLANK_INDENT,
                });
            }

            self.draw_branch(writer, de.path(), last, &idt);

            indents.push(if last { Indent::Blank } else { Indent::Bar });
            self.draw_from(writer, child, indents);
        }

        indents.pop();
    }

    fn draw_branch<W: Write>(&self, writer: &mut W, entry: &Path, last: bool, prefix: &str) {
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

// pub fn corners() -> ((u16, u16), (u16, u16)) {
//     let mut stdout = ::std::io::stdout().into_raw_mode().unwrap();

//     let (_, _tl_y) = stdout.cursor_pos().unwrap();
//     let (ogn_x, ogn_y) = (1, _tl_y - 1); // Termion and Tui aren't playing nice

//     let (_br_x, _br_y) = terminal_size().unwrap();
//     let (far_x, far_y) = (_br_x - ogn_x, _br_y - ogn_y);

//     ((ogn_x, ogn_y), (far_x, far_y))
// }

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
        Tree::new(dir).draw(&mut actual);

        String::from_utf8(actual).unwrap()
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
