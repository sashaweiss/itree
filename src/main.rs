extern crate ignore;
extern crate indextree;
extern crate termion;

mod fs;
mod term;
mod tree;

fn main() {
    let mut args = ::std::env::args();
    args.next();

    let root = match args.next() {
        Some(dir) => dir,
        None => String::from("."),
    };

    let mut t = tree::Tree::new(&root);
    term::navigate(&mut t);
}
