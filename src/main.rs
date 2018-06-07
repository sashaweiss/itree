extern crate ignore;
extern crate indextree;
extern crate termion;

mod fs;
mod tree;

fn main() {
    let t = tree::Tree::new(&".");
    // print!("{}", t);

    t.render_to_term(0).unwrap();

    // The following is necessary to properly read from stdin.
    // For details, see: https://github.com/ticki/termion/issues/42
    // let _stdout = io::stdout().into_raw_mode().unwrap();
    // fs::navigate(io::stdin(), cursor::new_cursor_bound_to_term(), tree, root);
}
