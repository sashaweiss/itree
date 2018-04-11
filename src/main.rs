use std::fs;

fn main() {
    let des = files_in_dir(&"./".to_string());
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
