use std::fmt;
use std::path::Path;

use termion::color::{self, Color};

pub struct RenderOptions {
    fg_color: Box<Color>,
    bg_color: Box<Color>,
}

impl fmt::Debug for RenderOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ fg_color: ?, bg_color: ? }}")
    }
}

impl RenderOptions {
    pub fn new() -> Self {
        Self {
            fg_color: Box::new(color::White),
            bg_color: Box::new(color::Blue),
        }
    }
}

#[derive(Debug)]
pub struct FsOptions {
    pub max_depth: Option<usize>,
    pub follow_links: bool,
    pub max_filesize: Option<u64>,
    pub hidden: bool,
    pub use_ignore: bool,
    pub use_git_exclude: bool,
    pub custom_ignore: Vec<String>,
}

impl FsOptions {
    pub fn new() -> Self {
        Self {
            max_depth: None,
            follow_links: false,
            max_filesize: None,
            hidden: true,
            use_ignore: true,
            use_git_exclude: true,
            custom_ignore: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TreeOptions<P: AsRef<Path>> {
    pub(crate) root: P,
    pub(crate) fs_opts: FsOptions,
    pub(crate) render_opts: RenderOptions,
}

impl<P: AsRef<Path>> TreeOptions<P> {
    pub fn new(root: P) -> Self {
        Self {
            root,
            fs_opts: FsOptions::new(),
            render_opts: RenderOptions::new(),
        }
    }

    /// Set the root directory from which to build the tree.
    pub fn root(&mut self, root: P) -> &mut Self {
        self.root = root;
        self
    }

    /// Set a maximum depth for the tree to search. `None` indicates no limit.
    ///
    /// `None` by default.
    pub fn max_depth(&mut self, max_depth: Option<usize>) -> &mut Self {
        self.fs_opts.max_depth = max_depth;
        self
    }

    /// Set whether or not to follow links.
    ///
    /// Disabled by default.
    pub fn follow_links(&mut self, follow_links: bool) -> &mut Self {
        self.fs_opts.follow_links = follow_links;
        self
    }

    /// Set a maximum file size to include. `None` indicates no limit.
    ///
    /// `None` by default.
    pub fn max_filesize(&mut self, max_filesize: Option<u64>) -> &mut Self {
        self.fs_opts.max_filesize = max_filesize;
        self
    }

    /// Set whether or not to ignore hidden files.
    ///
    /// Enabled by default.
    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.fs_opts.hidden = hidden;
        self
    }

    /// Set whether or not to read `.[git]ignore` files.
    ///
    /// Enabled by default.
    pub fn use_ignore(&mut self, use_ignore: bool) -> &mut Self {
        self.fs_opts.use_ignore = use_ignore;
        self
    }

    /// Set whether or not to read `.git/info/exclude` files.
    ///
    /// Enabled by default.
    pub fn use_git_exclude(&mut self, use_git_exclude: bool) -> &mut Self {
        self.fs_opts.use_git_exclude = use_git_exclude;
        self
    }

    /// Add a custom ignore path.
    pub fn add_custom_ignore(&mut self, path: &str) -> &mut Self {
        self.fs_opts.custom_ignore.push(path.to_owned());
        self
    }
}
