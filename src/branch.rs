use std::iter;
use std::path::Path;
use std::io::Write;

use ignore::{WalkBuilder, Walk, DirEntry};
use indextree::{Arena, NodeId};

pub const MID_BRANCH: &str = "├──";
pub const END_BRANCH: &str = "└──";
pub const INDENT: &str = "    ";

/// Create an iterator over the FS, rooted at dir.
fn get_walker<P: AsRef<Path>>(dir: &P) -> iter::Peekable<Walk> {
    WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .build()
        .peekable()
}

/// Add a node to `tree`, as a child of `node`, with `data` as the contents.
fn add_child_to_tree<T>(tree: &mut Arena<T>, node: NodeId, data: T) -> NodeId {
    let new_node = tree.new_node(data);
    if let Some(p) = tree[node].parent() {
        p.append(new_node, tree); // curr != root
    } else {
        node.append(new_node, tree); // curr == root
    }
    new_node
}

/// Collect an Arena representation of the file system.
fn collect_fs<P: AsRef<Path>>(dir: &P) -> (Arena<DirEntry>, NodeId) {
    let mut walk = get_walker(dir);

    let mut tree = Arena::<DirEntry>::new();
    let root: NodeId;

    // Get the root node
    if let Some(Ok(de)) = walk.next() {
        root = tree.new_node(de);
    } else {
        panic!("Failed to get the root!");
    }

    enum DepthChange {
        NextIsFirst,
        Isnt,
        Last(usize),
    }

    let mut curr = root;
    while let Some(Ok(de)) = walk.next() {
        match match walk.peek() {
            Some(&Ok(ref next)) => {
                let (nd, dd) = (next.depth(), de.depth());

                if nd > dd {
                    DepthChange::Last(nd - dd)
                } else if nd < dd {
                    DepthChange::NextIsFirst
                } else {
                    DepthChange::Isnt
                }
            }
            Some(&Err(_)) => continue,
            None => DepthChange::Last(0),
        } {
            DepthChange::NextIsFirst => {
                curr = add_child_to_tree(&mut tree, curr, de);
            }
            DepthChange::Isnt => {
                add_child_to_tree(&mut tree, curr, de);
            }
            DepthChange::Last(d) => {
                add_child_to_tree(&mut tree, curr, de);
                for _ in 1..d {
                    curr = tree[curr].parent().expect("The node should have a parent");
                }
            }
        }
    }

    (tree, root)
}

/// Draw the tree, starting with the given directory.
pub fn draw_rooted<W: Write, P: AsRef<Path>>(_writer: &mut W, dir: &P) {
    // (tree, root) = collect_fs(dir);

    // draw_root(writer, dir.as_ref());

    // draw_branch(writer, &de.path(), last, de.depth() - 1);
}

fn draw_branch<W: Write>(writer: &mut W, entry: &Path, last: bool, indent: usize) {
    let file_name = match entry.file_name() {
        Some(name) => name,
        None => return,
    }.to_string_lossy();

    writeln!(
        writer,
        "{}{} {}",
        iter::repeat(INDENT).take(indent).collect::<String>(),
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
                "myotherfile",
                END_BRANCH,
                "myfile",
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
                "myotherfile",
                END_BRANCH,
                "myfile",
            );

        assert_eq!(exp, draw_to_string(&dir));
    }

    #[test]
    fn test_draw_dir() {
        let dir = test_dir("one_dir");

        let exp =
            format!(
            "{}\n{} {}\n{} {}\n{}{} {}\n",
            dir.display(),
            MID_BRANCH,
            "myotherfile",
            END_BRANCH,
            "mydir",
            INDENT,
            END_BRANCH,
            "myfile",
         );

        assert_eq!(exp, draw_to_string(&dir));
    }
}
