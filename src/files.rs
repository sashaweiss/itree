use std::fs;
use std::io;
use std::cmp::Ordering;

pub fn in_root() -> Vec<fs::DirEntry> {
    in_dir(&"./".to_string()).unwrap()
}

pub fn in_dir(dir: &str) -> io::Result<Vec<fs::DirEntry>> {
    let mut des: Vec<fs::DirEntry> = fs::read_dir(dir)?
        .filter(|de| de.is_ok())
        .map(|de| de.unwrap())
        .collect();

    des.sort_by(|f: &fs::DirEntry, s: &fs::DirEntry| -> Ordering {
        f.path().cmp(&s.path())
    });

    Ok(des)
}
