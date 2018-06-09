use std::ffi::OsStr;
use std::iter;
use std::path::Path;

use tree::TreeOptions;

use ignore::{DirEntry, Walk, WalkBuilder, overrides::OverrideBuilder};
use indextree::{Arena, NodeId};

#[derive(Debug)]
pub struct FsEntry {
    pub de: DirEntry,
    pub name: String,
}

/// Create an iterator over the FS, rooted at dir.
fn get_walker<P: AsRef<Path>>(options: &TreeOptions<P>) -> Result<iter::Peekable<Walk>, String> {
    let dir = &options.root;
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
        ovs.add(&file)
            .map_err(|e| format!("Error adding custom ignore glob: {:?}", e))?;
    }

    builder.overrides(ovs.build()
        .map_err(|e| format!("Error adding custom ignore glob: {:?}", e))?);

    Ok(builder.build().peekable())
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

/// Collect an Arena representation of the file system.
pub fn fs_to_tree<P: AsRef<Path>>(
    options: TreeOptions<P>,
) -> Result<(Arena<FsEntry>, NodeId), String> {
    let dir = &options.root;
    let mut walk = get_walker(&options)?;

    let mut tree = Arena::<FsEntry>::new();
    let root: NodeId;

    // Get the root node
    if let Some(Ok(de)) = walk.next() {
        root = tree.new_node(FsEntry {
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
        });
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
        let te = FsEntry {
            name: path_to_string(&de.path()),
            de: de,
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
    }

    Ok((tree, root))
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
        let (tree, root) = fs_to_tree(TreeOptions::new(&abs_test_dir("simple"))).unwrap();
        assert_eq!("simple", tree[root].data.de.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.de.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("myfile")));
        assert!(children.contains(&to_osstr("myotherfile")));
    }

    #[test]
    fn test_collect_fs_rel_path() {
        let (tree, root) = fs_to_tree(TreeOptions::new(&test_dir("simple"))).unwrap();
        assert_eq!("simple", tree[root].data.de.file_name());

        let children = root.children(&tree)
            .map(|nid| tree[nid].data.de.path().file_name().unwrap())
            .collect::<Vec<&OsStr>>();

        assert!(children.contains(&to_osstr("myfile")));
        assert!(children.contains(&to_osstr("myotherfile")));
    }

    #[test]
    fn test_collect_fs_dir() {
        let (tree, root) = fs_to_tree(TreeOptions::new(&test_dir("one_dir"))).unwrap();
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
