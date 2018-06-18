use std::ffi::OsStr;
use std::fs::read_link;
use std::io;
use std::iter;
use std::ops::Deref;
use std::path::Path;

use options::FsOptions;

use ignore::{self, DirEntry, Walk, WalkBuilder, overrides::OverrideBuilder};
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
        .ignore(!options.no_ignore)
        .git_global(!options.no_ignore)
        .git_ignore(!options.no_ignore)
        .git_exclude(!options.no_git_exclude);

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

enum DepthChange {
    NextIsFirst,
    Isnt,
    Last(usize),
}

impl DepthChange {
    fn for_depths(next: usize, curr: usize) -> Self {
        if next < curr {
            DepthChange::Last(curr - next)
        } else if next > curr {
            DepthChange::NextIsFirst
        } else {
            DepthChange::Isnt
        }
    }
}

fn determine_place_in_tree(walk: &mut iter::Peekable<Walk>, fse: &mut FsEntry) -> DepthChange {
    match walk.peek() {
        Some(Ok(next)) => return DepthChange::for_depths(next.depth(), fse.de.depth()),
        None => return DepthChange::Last(0),
        Some(Err(e)) => {
            let mut ok = false;
            if let ignore::Error::WithPath { path, err } = e {
                if let ignore::Error::Io(inner) = err.deref() {
                    ok = inner.kind() == io::ErrorKind::PermissionDenied && path == fse.de.path();
                }
            }

            if ok {
                // A permission-denied error trying to recur into a subdirectory.
                // This is fine - it does mean we want to add that information to the current
                // FsEntry. See https://github.com/BurntSushi/ripgrep/issue/953.
                fse.name.push_str(" [error opening dir]");
            } else {
                eprintln!("Error while building FS tree: {:?}", e);
            }
        }
    };

    walk.next();
    determine_place_in_tree(walk, fse)
}

/// Collect an Arena representation of the file system.
pub fn fs_to_tree<P: AsRef<Path>>(
    dir: &P,
    options: &FsOptions,
) -> (Arena<FsEntry>, NodeId, usize, usize) {
    let mut walk = get_walker(dir, &options);

    let mut tree = Arena::<FsEntry>::new();
    let root = if let Some(Ok(de)) = walk.next() {
        tree.new_node(root_to_fsentry(dir, de))
    } else {
        panic!("Failed to get the root!");
    };

    let mut n_files = 0;
    let mut n_dirs = 0;
    let mut curr = root;
    while let Some(res) = walk.next() {
        let mut fse = match res {
            Ok(de) => {
                if let Some(ft) = de.file_type() {
                    if ft.is_dir() {
                        n_dirs += 1;
                    } else {
                        n_files += 1;
                    }
                }

                de_to_fsentry(de)
            }
            Err(_) => {
                panic!("This error should have been handled in `determine_place_in_tree` during peeking.");
            }
        };

        match determine_place_in_tree(&mut walk, &mut fse) {
            DepthChange::NextIsFirst => {
                curr = add_child_to_tree(&mut tree, curr, fse);
            }
            DepthChange::Isnt => {
                add_child_to_tree(&mut tree, curr, fse);
            }
            DepthChange::Last(d) => {
                add_child_to_tree(&mut tree, curr, fse);
                for _ in 0..d {
                    curr = tree[curr].parent().expect("The node should have a parent");
                }
            }
        }
    }

    (tree, root, n_files, n_dirs)
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
        let (tree, root, _, _) = fs_to_tree(dir, &FsOptions::new());
        (tree, root)
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
