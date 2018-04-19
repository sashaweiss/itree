use std::iter;
use std::path::Path;

use ignore::{WalkBuilder, Walk, DirEntry};
use indextree::{Arena, NodeId};

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
pub fn collect_fs<P: AsRef<Path>>(dir: &P) -> (Arena<DirEntry>, NodeId) {
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
