use std::iter;
use std::path::Path;

use ignore::{DirEntry, Walk, WalkBuilder};
use indextree::{Arena, NodeId};

#[derive(Debug)]
pub struct TreeEntry {
    pub de: DirEntry,
    pub loc: (usize, usize),
}

/// Create an iterator over the FS, rooted at dir.
fn get_walker<P: AsRef<Path>>(dir: &P) -> iter::Peekable<Walk> {
    WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .sort_by_file_name(|f1, f2| f1.cmp(f2))
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
pub fn collect_fs<P: AsRef<Path>>(dir: &P) -> (Arena<TreeEntry>, NodeId) {
    let mut walk = get_walker(dir);

    let mut tree = Arena::<TreeEntry>::new();
    let root: NodeId;

    // Get the root node
    if let Some(Ok(de)) = walk.next() {
        root = tree.new_node(TreeEntry { de, loc: (0, 0) });
    } else {
        panic!("Failed to get the root!");
    }

    enum DepthChange {
        NextIsFirst,
        Isnt,
        Last(usize),
    }

    let mut curr = root;
    let mut n_seen = 0;
    while let Some(Ok(de)) = walk.next() {
        let te = TreeEntry {
            loc: (de.depth(), n_seen),
            de,
        };

        match match walk.peek() {
            Some(&Ok(ref next)) => {
                let (nd, dd) = (next.depth(), te.de.depth());

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
                curr = add_child_to_tree(&mut tree, curr, te);
            }
            DepthChange::Isnt => {
                add_child_to_tree(&mut tree, curr, te);
            }
            DepthChange::Last(d) => {
                add_child_to_tree(&mut tree, curr, te);
                for _ in 0..d {
                    curr = tree[curr].parent().expect("The node should have a parent");
                }
            }
        }

        n_seen += 1;
    }

    (tree, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsStr;
    use std::path::PathBuf;

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
        assert_eq!("simple", tree[root].data.de.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.de.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("myfile")));
        assert!(children.contains(&to_osstr("myotherfile")));
    }

    #[test]
    fn test_collect_fs_rel_path() {
        let (tree, root) = collect_fs(&test_dir("simple"));
        assert_eq!("simple", tree[root].data.de.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.de.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("myfile")));
        assert!(children.contains(&to_osstr("myotherfile")));
    }

    #[test]
    fn test_collect_fs_dir() {
        let (tree, root) = collect_fs(&test_dir("one_dir"));
        assert_eq!("one_dir", tree[root].data.de.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.de.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("mydir")));
        assert!(children.contains(&to_osstr("myotherfile")));

        let dir_node = root.children(&tree)
            .filter(|nid| tree[*nid].data.de.path().file_name() == Some(to_osstr("mydir")))
            .next()
            .unwrap();

        assert_eq!(
            tree[dir_node.children(&tree).next().unwrap()]
                .data
                .de
                .path()
                .file_name(),
            Some(to_osstr("myfile"))
        );
    }
}
