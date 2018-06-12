#![feature(test)]

extern crate test;
extern crate rusty_tree;

use test::Bencher;

#[bench]
fn bench_home_dir(b: &mut Bencher) {
    b.iter(|| {
        rusty_tree::tree::Tree::new_from_dir(&String::from("/Users/sasha"));
    });
}

#[bench]
fn bench_curr_dir(b: &mut Bencher) {
    b.iter(|| {
        rusty_tree::tree::Tree::new();
    });
}
