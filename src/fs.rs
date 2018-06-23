use std::ffi::OsStr;
use std::fs::{metadata, read_link};
use std::io;
use std::ops::Deref;
use std::path::Path;

use options::FsOptions;
use util::PutBack;

use ignore::{self, overrides::OverrideBuilder, DirEntry, Walk, WalkBuilder};
use indextree::{Arena, NodeId};

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
    RestrictedDir,
    Stdin,
    LinkTo(String),
}

#[derive(Debug)]
pub struct FsEntry {
    pub ft: FileType,
    pub de: DirEntry,
    pub name: String,
}

/// Create an iterator over the FS, rooted at dir.
fn get_walker<P: AsRef<Path>>(options: &FsOptions<P>) -> Walk {
    let mut builder = WalkBuilder::new(&options.root);

    builder
        .parents(false)
        .sort_by_file_name(|f1, f2| f1.cmp(f2))
        .follow_links(options.follow_links)
        .max_filesize(options.max_filesize)
        .hidden(!options.hidden)
        .ignore(!options.no_ignore)
        .git_global(!options.no_ignore)
        .git_ignore(!options.no_ignore)
        .git_exclude(!options.no_git_exclude);

    let mut ovs = OverrideBuilder::new(&options.root);
    for file in options.custom_ignore.iter() {
        ovs.add(&file).unwrap();
    }

    builder.overrides(ovs.build().unwrap());

    builder.build()
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
    let name = path_to_string(&de.path());
    let ft = if de.path_is_symlink() {
        let dest = match read_link(&de.path()) {
            Ok(d) => path_to_string(&d),
            Err(_) => "<error reading dest>".to_owned(),
        };

        FileType::LinkTo(dest)
    } else {
        match de.file_type() {
            Some(t) => if t.is_dir() {
                FileType::Dir
            } else {
                FileType::File
            },
            None => FileType::Stdin,
        }
    };

    FsEntry { ft, de, name }
}

fn root_to_fsentry<P: AsRef<Path>>(dir: &P, de: DirEntry) -> FsEntry {
    FsEntry {
        ft: FileType::Dir,
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

fn is_or_points_to_dir(de: &DirEntry) -> bool {
    match de.file_type() {
        Some(ft) => {
            if ft.is_dir() {
                true
            } else if ft.is_file() {
                false
            } else {
                match metadata(&de.path()) {
                    Ok(meta) => meta.is_dir(),
                    Err(_) => false,
                }
            }
        }
        None => false, // This means stdin.
    }
}

fn determine_place_in_tree(
    walk: &mut PutBack<Walk>,
    fse: &mut FsEntry,
    only_dirs: bool,
) -> DepthChange {
    while let Some(next) = walk.next() {
        match next {
            Ok(next) => {
                if only_dirs && !is_or_points_to_dir(&next) {
                    continue;
                }

                let nd = next.depth();
                walk.put_back(Ok(next));
                return DepthChange::for_depths(nd, fse.de.depth());
            }
            Err(ref e) => {
                if let ignore::Error::WithPath { path, err } = e {
                    if let ignore::Error::Io(inner) = err.deref() {
                        if inner.kind() == io::ErrorKind::PermissionDenied && path == fse.de.path()
                        {
                            // A permission-denied error trying to recur into a subdirectory.
                            // This is fine, but we want to keep track of it.
                            // See https://github.com/BurntSushi/ripgrep/issue/953.
                            fse.ft = FileType::RestrictedDir;
                            continue;
                        }
                    }
                }

                eprintln!("Unexpected error while building tree.\nDetails: {:?}", e);
            }
        }
    }

    DepthChange::Last(0)
}

/// Collect an Arena representation of the file system.
///
/// Returns an Arena-tree, its root, and the number of files
/// and directories in it.
pub fn fs_to_tree<P: AsRef<Path>>(
    options: &FsOptions<P>,
) -> (Arena<FsEntry>, NodeId, usize, usize) {
    let mut walk = PutBack::new(get_walker(&options));

    let mut tree = Arena::<FsEntry>::new();
    let root = match walk.next() {
        Some(Ok(de)) => tree.new_node(root_to_fsentry(&options.root, de)),
        Some(Err(ignore::Error::WithPath { path, err })) => {
            if let ignore::Error::Io(e) = err.deref() {
                if e.kind() == io::ErrorKind::NotFound {
                    eprintln!("Path \"{}\" was not found", path.display());
                } else {
                    eprintln!(
                        "An error occurred getting {}: {:?}",
                        path.display(),
                        e.kind()
                    );
                }
            } else {
                eprintln!("Could not get {}: {:?}", path.display(), err);
            }

            ::std::process::exit(1);
        }
        Some(Err(_)) => {
            eprintln!("An unexpected error occurred getting the root!");
            ::std::process::exit(2);
        }
        None => {
            unreachable!("Should always get a first walk entry");
        }
    };

    let mut n_files = 0;
    let mut n_dirs = 0;
    let mut curr = root;
    while let Some(res) = walk.next() {
        let mut fse = match res {
            Ok(de) => {
                if is_or_points_to_dir(&de) {
                    n_dirs += 1;
                } else if options.only_dirs {
                    continue;
                } else {
                    n_files += 1;
                }

                de_to_fsentry(de)
            }
            Err(_) => {
                panic!("This error should have been handled in `determine_place_in_tree` in the previous iteration.");
            }
        };

        match determine_place_in_tree(&mut walk, &mut fse, options.only_dirs) {
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
        let (tree, root, _, _) = fs_to_tree(&FsOptions::new(dir));
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
}
