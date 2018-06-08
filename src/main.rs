extern crate ignore;
extern crate indextree;
extern crate termion;

mod fs;
mod term;
mod tree;

fn main() {
    let mut t = tree::Tree::new(&".");
    term::navigate(&mut t);
}
