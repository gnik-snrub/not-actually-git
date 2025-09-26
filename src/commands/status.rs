use crate::core::{
    repo::find_repo_root,
    index::read_index,
    io::read_file,
};
use crate::commands::hash::hash;

use std::fs::read_dir;
use std::path::Path;
use std::collections::{ HashSet, HashMap };

pub fn status() -> std::io::Result<()> {
    let index = read_index()?;
    let mut working: Vec<(String, String)> = vec![];
    let root = find_repo_root()?;

    walk(&root, &mut working, &root)?;

    let mut untracked = vec![];
    let mut modified = vec![];
    let mut staged = vec![];
    let mut deleted = vec![];

    let index_map: HashMap<String, String> = index.iter()
        .map(|(oid, p)| (p.clone(), oid.clone()))
        .collect();
    let wrk_paths: HashSet<_> = working.iter().map(|(_, p)| p.clone()).collect();

    for working_entry in working { 
        let wrk_oid = working_entry.0;
        let wrk_path = working_entry.1;
        if let Some(index_oid) = index_map.get(&wrk_path) {
            if &wrk_oid != index_oid {
                modified.push(wrk_path);
            } else {
                staged.push(wrk_path);
            }
        } else {
            untracked.push(wrk_path);
        }
    }

    for index_entry in index {
        let index_entry_path = index_entry.1;
        if !wrk_paths.contains(&index_entry_path) {
            deleted.push(index_entry_path);
        }
    }

    if untracked.len() > 0 {
        println!("\nUntracked files");
        for path in untracked {
            println!("\t{}", path);
        }
    }

    if deleted.len() > 0 || modified.len() > 0 {
        println!("\nChanges not staged for commit");
    }

    if deleted.len() > 0 {
        println!("\nDeleted");
        for path in deleted {
            println!("\t{}", path);
        }
    }
    if modified.len() > 0 {
        println!("\nModified");
        for path in modified {
            println!("\t{}", path);
        }
    }

    if staged.len() > 0{
        println!("\nStaged changes");
        for path in staged {
            println!("\t{}", path);
        }
    }

    Ok(())
}

fn walk(path: &Path, working: &mut Vec<(String, String)>, root: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for child in read_dir(path)? {
            let dir = child.unwrap();
            walk(&dir.path(), working, root)?;
        }
    } else if path.is_file() {
        let abs_path = path.canonicalize()?;

        let rel_path = path.strip_prefix(&root).unwrap_or(path);
        let mut rel_str = rel_path.to_string_lossy().to_string();

        rel_str = rel_str.replace('\\', "/");
        if rel_str.starts_with("./") {
            rel_str = rel_str[2..].to_string();
        }

        let file = read_file(&abs_path.to_string_lossy());
        let blob = hash(&file);
        working.push((blob, rel_str));
    }
    Ok(())
}
