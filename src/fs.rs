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
    node.append(new_node, tree);
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

                if nd < dd {
                    DepthChange::Last(dd - nd)
                } else if nd > dd {
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
                for _ in 0..d {
                    curr = tree[curr].parent().expect("The node should have a parent");
                }
            }
        }
    }

    (tree, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::ffi::OsStr;

    fn test_dir(dir: &str) -> PathBuf {
        PathBuf::new().join("resources/test").join(dir)
    }

    fn abs_test_dir(dir: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(test_dir(dir))
    }

    fn to_osstr(s: &str) -> &OsStr {
        OsStr::new(s)
    }

    #[test]
    fn test_collect_fs_abs_path() {
        let (tree, root) = collect_fs(&abs_test_dir("simple"));
        assert_eq!("simple", tree[root].data.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("myfile")));
        assert!(children.contains(&to_osstr("myotherfile")));
    }

    #[test]
    fn test_collect_fs_rel_path() {
        let (tree, root) = collect_fs(&test_dir("simple"));
        assert_eq!("simple", tree[root].data.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("myfile")));
        assert!(children.contains(&to_osstr("myotherfile")));
    }

    #[test]
    fn test_collect_fs_dir() {
        let (tree, root) = collect_fs(&test_dir("one_dir"));
        assert_eq!("one_dir", tree[root].data.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("mydir")));
        assert!(children.contains(&to_osstr("myotherfile")));

        let dir_node = root.children(&tree)
            .filter(|nid| {
                tree[*nid].data.path().file_name() == Some(to_osstr("mydir"))
            })
            .next()
            .unwrap();

        assert_eq!(
            tree[dir_node.children(&tree).next().unwrap()]
                .data
                .path()
                .file_name(),
            Some(to_osstr("myfile"))
        );
    }
}
