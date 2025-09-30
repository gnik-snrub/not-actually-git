use std::path::Path;
use std::fs::read_dir;

use crate::core::io::{ read_file, write_object };
use crate::core::index::{ read_index, write_index };
use crate::core::hash::hash;
use crate::core::repo::find_repo_root;

pub fn add(path: &Path) -> std::io::Result<()> {
    let mut index = read_index()?;
    walk(path, &mut index)?;
    write_index(&index)?;
    Ok(())
}

fn walk(path: &Path, entries: &mut Vec<(String, String)>) -> std::io::Result<()> {
    if path.is_dir() {
        for child in read_dir(path)? {
            let dir = child.unwrap();
            walk(&dir.path(), entries)?;
        }
    } else if path.is_file() {
        let abs_path = path.canonicalize()?;

        let repo_root = find_repo_root()?;
        let rel_path = path.strip_prefix(&repo_root).unwrap_or(path);
        let mut rel_str = rel_path.to_string_lossy().to_string();

        rel_str = rel_str.replace('\\', "/");
        if rel_str.starts_with("./") {
            rel_str = rel_str[2..].to_string();
        }

        let file = read_file(&abs_path.to_string_lossy());
        let blob = hash(&file);
        write_object(&file, &blob)?;
        update_or_insert(blob, rel_str, entries);
    }
    Ok(())
}

fn update_or_insert(oid: String, path: String, entries: &mut Vec<(String, String)>) {
    let mut found = false;
    for entry in &mut *entries {
        if entry.1 == path {
            entry.0 = oid.clone();
            found = true;
        }
    }
    if !found {
        entries.push((oid, path));
    }
}
