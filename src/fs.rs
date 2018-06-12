use std::ffi::OsStr;
use std::fs::read_link;
use std::iter;
use std::path::Path;

use options::FsOptions;

use ignore::{DirEntry, Walk, WalkBuilder, overrides::OverrideBuilder};
use indextree::{Arena, NodeId};

#[derive(Debug)]
pub struct FsEntry {
    pub de: DirEntry,
    pub name: String,
}

/// Create an iterator over the FS, rooted at dir.
fn get_walker<P: AsRef<Path>>(dir: &P, options: &FsOptions) -> iter::Peekable<Walk> {
    let mut builder = WalkBuilder::new(&dir);

    builder
        .parents(false)
        .sort_by_file_name(|f1, f2| f1.cmp(f2))
        .max_depth(options.max_depth)
        .follow_links(options.follow_links)
        .max_filesize(options.max_filesize)
        .hidden(!options.hidden)
        .ignore(!options.use_ignore)
        .git_global(!options.use_ignore)
        .git_ignore(!options.use_ignore)
        .git_exclude(!options.use_git_exclude);

    let mut ovs = OverrideBuilder::new(dir);
    for file in options.custom_ignore.iter() {
        ovs.add(&file).unwrap();
    }

    builder.overrides(ovs.build().unwrap());

    builder.build().peekable()
}

/// Add a node to `tree`, as a child of `node`, with `data` as the contents.
fn add_child_to_tree<T>(tree: &mut Arena<T>, node: NodeId, data: T) -> NodeId {
    let new_node = tree.new_node(data);
    node.append(new_node, tree);
    new_node
}

fn path_to_string<P: AsRef<Path>>(p: &P) -> String {
    match p.as_ref().file_name() {
        Some(name) => name.to_str().unwrap_or("<node name non-UTF8"),
        None => "<node name unknown>",
    }.to_owned()
}

fn de_to_fsentry(de: DirEntry) -> FsEntry {
    let mut name = path_to_string(&de.path());
    if de.path_is_symlink() {
        let dest = match read_link(&de.path()) {
            Ok(d) => path_to_string(&d),
            Err(_) => "<error reading dest>".to_owned(),
        };

        name.push_str(" -> ");
        name.push_str(&dest);
    }

    FsEntry { de, name }
}

fn root_to_fsentry<P: AsRef<Path>>(dir: &P, de: DirEntry) -> FsEntry {
    FsEntry {
        de,
        name: if dir.as_ref() == OsStr::new(".") {
            ".".to_owned()
        } else {
            let mut d = format!("{}", dir.as_ref().display());
            if d.ends_with("/") {
                d.pop();
            }
            d
        },
    }
}

/// Collect an Arena representation of the file system.
pub fn fs_to_tree<P: AsRef<Path>>(dir: &P, options: &FsOptions) -> (Arena<FsEntry>, NodeId) {
    let mut walk = get_walker(dir, &options);

    let mut tree = Arena::<FsEntry>::new();
    let root: NodeId;

    // Get the root node (never a symlink)
    if let Some(Ok(de)) = walk.next() {
        root = tree.new_node(root_to_fsentry(dir, de));
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
        let te = de_to_fsentry(de);

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
    }

    (tree, root)
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

    fn test_tree(dir: &PathBuf) -> (Arena<FsEntry>, NodeId) {
        fs_to_tree(dir, &FsOptions::new())
    }

    #[test]
    fn test_collect_fs_abs_path() {
        let (tree, root) = test_tree(&abs_test_dir("simple"));

        let mut curr = ::std::env::current_dir().unwrap();
        curr.push("resources/test/simple");
        assert_eq!(
            format!("{}", curr.as_path().display()),
            tree[root].data.name
        );

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.name.as_str())
            .collect::<Vec<&str>>();

        assert!(children.contains(&"myfile"));
        assert!(children.contains(&"myotherfile"));
    }

    #[test]
    fn test_collect_fs_rel_path() {
        let (tree, root) = test_tree(&test_dir("simple"));
        assert_eq!(tree[root].data.name, "resources/test/simple");

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.name.as_str())
            .collect::<Vec<&str>>();

        assert!(children.contains(&"myfile"));
        assert!(children.contains(&"myotherfile"));
    }

    #[test]
    fn test_collect_fs_dir() {
        let (tree, root) = test_tree(&test_dir("one_dir"));
        assert_eq!(tree[root].data.name, "resources/test/one_dir");

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.name.as_str())
            .collect::<Vec<&str>>();

        assert!(children.contains(&"mydir"));
        assert!(children.contains(&"myotherfile"));

        let dir_node = root.children(&tree)
            .filter(|nid| tree[*nid].data.name.as_str() == "mydir")
            .next()
            .unwrap();

        assert_eq!(
            tree[dir_node.children(&tree).next().unwrap()].data.name,
            "myfile",
        );
    }

    #[test]
    fn test_link_fsentry() {
        let (tree, root) = test_tree(&test_dir("link"));
        assert_eq!("resources/test/link", tree[root].data.name);

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.name.as_str())
            .collect::<Vec<&str>>();

        assert!(children.contains(&"source"));
        assert!(children.contains(&"dest -> source"));
    }
}
