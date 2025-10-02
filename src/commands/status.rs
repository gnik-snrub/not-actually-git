use crate::core::{
    repo::find_repo_root,
    index::read_index,
    io::read_file,
};
use crate::core::hash::hash;
use crate::core::tree::read_tree_to_index;

use std::fs::read_dir;
use std::path::Path;
use std::collections::{ HashSet, HashMap };

pub fn status() -> std::io::Result<String> {
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

    let head_path = root.join(".nag").join("HEAD");
    let proj_head_contents = read_file(&head_path.to_string_lossy());
    let head_str = String::from_utf8_lossy(&proj_head_contents);

    let target = head_str.trim();
    let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
    let branch_path = root.join(".nag").join(branch_path_fragment);
    let branch_contents = read_file(&branch_path.to_string_lossy());
    let branch_oid = String::from_utf8_lossy(&branch_contents);

    let head_index_map = if branch_oid.trim().is_empty() {
        HashMap::new()
    } else {
        let commit_path = root.join(".nag").join("objects").join(format!("{}", branch_oid));
        let commit_contents = read_file(&commit_path.to_string_lossy());
        let commit_str = String::from_utf8_lossy(&commit_contents);

        let tree_line = commit_str.lines().collect::<Vec<&str>>()[0];
        let tree_oid = tree_line.split(" ").collect::<Vec<&str>>()[1];
        let head_index = read_tree_to_index(&tree_oid)?;

        head_index.into_iter()
            .map(|(oid, p)| (p.clone(), oid.clone()))
            .collect()
    };


    for working_entry in working { 
        let wrk_oid = working_entry.0;
        let wrk_path = working_entry.1;
        if let Some(index_oid) = index_map.get(&wrk_path) {
            if &wrk_oid != index_oid {
                modified.push(wrk_path);
            }
        } else {
            untracked.push(wrk_path);
        }
    }

    for index_entry in index {
        let index_entry_oid = index_entry.0;
        let index_entry_path = index_entry.1;
        if !wrk_paths.contains(&index_entry_path) {
            deleted.push(index_entry_path.clone());
        }
        if let Some(head_entry_oid) = head_index_map.get(&index_entry_path) {
            if *head_entry_oid != index_entry_oid {
                staged.push(index_entry_path);
            }
        } else {
            staged.push(index_entry_path);
        }
    }

    let mut buf_str = String::new();

    if untracked.len() > 0 {
        buf_str.push_str("\nUntracked files\n");
        for path in untracked {
            buf_str.push_str(&format!("\t{}\n", path));
        }
    }

    if deleted.len() > 0 || modified.len() > 0 {
        buf_str.push_str("\nChanges not staged for commit\n");
    }

    if deleted.len() > 0 {
        buf_str.push_str("\nDeleted\n");
        for path in deleted {
            buf_str.push_str(&format!("\t{}\n", path));
        }
    }
    if modified.len() > 0 {
        buf_str.push_str("\nModified\n");
        for path in modified {
            buf_str.push_str(&format!("\t{}\n", path));
        }
    }

    if staged.len() > 0{
        buf_str.push_str("\nStaged changes\n");
        for path in staged {
            buf_str.push_str(&format!("\t{}\n", path));
        }
    }

    println!("{buf_str}");

    Ok(buf_str)
}

fn walk(path: &Path, working: &mut Vec<(String, String)>, root: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for child in read_dir(path)? {
            let dir = child.unwrap();
            if path.file_name().map_or(false, |n| n == ".nag") {
                return Ok(());
            }
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
