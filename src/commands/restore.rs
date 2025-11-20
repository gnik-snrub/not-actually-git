use crate::core::repo::find_repo_root;
use crate::core::io::{ read_file, write_file };
use crate::core::tree::read_tree_to_index;
use crate::core::index::IndexEntry;

use std::collections::HashMap;
use std::fs::create_dir_all;

pub fn restore(restore_path: String) -> std::io::Result<()> {
    let root = find_repo_root()?;
    let nag_dir = root.join(".nag");

    let proj_head = nag_dir.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy())?;
    let head_str = String::from_utf8_lossy(&proj_head_contents);

    let target = head_str.trim();
    let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
    let branch_path = nag_dir.join(branch_path_fragment);
    let branch_contents = read_file(&branch_path.to_string_lossy())?;
    let branch_str = String::from_utf8_lossy(&branch_contents).trim().to_string();

    let commit_path = nag_dir.join("objects").join(branch_str.trim());
    let commit_contents = read_file(&commit_path.to_string_lossy())?;
    let commit_str = String::from_utf8_lossy(&commit_contents);

    let tree_line = commit_str.lines().next().unwrap();
    let tree_oid = tree_line.strip_prefix("tree ").unwrap().trim();

    let tree_map: HashMap<String, IndexEntry> = read_tree_to_index(&tree_oid)?
        .into_iter()
        .map(|entry| {
            return (entry.path.clone(), entry);
        })
        .collect();

    let mut restored_count = 0;
    let objects_dir = nag_dir.join("objects");
    println!("Restored:");
    for (path, entry) in &tree_map {
        if path == &restore_path || path.starts_with(&format!("{}/", restore_path)) {
            if entry.mode == "040000" {
                let dir_path = root.join(path);
                create_dir_all(&dir_path)?;
                continue;
            }

            let object_path = objects_dir.join(&entry.oids[0]);
            if !object_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Missing blob object for {}", path),
                ));
            }
            let object_contents = read_file(&object_path.to_string_lossy())?;
            write_file(&object_contents, &root.join(path))?;
            restored_count += 1;
            println!("\t{}", path);
        }
    }

    if restored_count == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No matches restored"),
        ));
    }

    Ok(())
}
