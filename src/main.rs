use std::fs;
use std::cmp::Ordering;

fn main() {
    let mut des = files_in_dir(&"./".to_string());

    des.sort_by(|f: &fs::DirEntry, s: &fs::DirEntry| -> Ordering {
        f.path().cmp(&s.path())
    });

    for de in des {
        println!("Path: {}", de.path().display());
    }
}

fn files_in_dir(dir: &str) -> Vec<fs::DirEntry> {
    fs::read_dir(dir)
        .unwrap()
        .filter(|de| de.is_ok())
        .map(|de| de.unwrap())
        .collect()
}
