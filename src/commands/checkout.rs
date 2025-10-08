use crate::commands::status::status;
use crate::core::io::read_file;
use crate::core::repo::find_repo_root;
use crate::core::tree::read_tree_to_index;
use crate::core::io::write_file;
use crate::core::index::write_index;

use std::fs::{ read_dir, remove_file, remove_dir_all, create_dir_all };
use std::path::Path;

pub fn checkout(branch: String) -> std::io::Result<()> {
    if status(false)?.len() > 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("There are un-committed changes made. Please save your changes before checkout"),
        ));
    }

    let root = find_repo_root()?;
    let nag_dir = root.join(".nag");
    let branch_path = nag_dir.join(format!("refs/heads/{}", branch));
    if !branch_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Branch '{}' not found", branch),
        ));
    }
    let branch_contents = read_file(&branch_path.to_string_lossy());
    let branch_str = String::from_utf8_lossy(&branch_contents);

    let commit_path = nag_dir.join("objects").join(branch_str.trim());
    if !commit_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit '{}' not found", branch),
        ));
    }
    let commit_contents = read_file(&commit_path.to_string_lossy());
    let commit_str = String::from_utf8_lossy(&commit_contents);

    let tree_line = commit_str.lines().next().unwrap();
    let tree_oid = tree_line.strip_prefix("tree ").unwrap().trim();
    let tree_path = nag_dir.join("objects").join(tree_oid.trim());
    if !tree_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit\'s tree '{}' not found", branch),
        ));
    }

    for entry in read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_name() == ".nag" {
            continue;
        }

        if path.is_dir() {
            remove_dir_all(&path)?;
        } else {
            remove_file(&path)?;
        }
    }

    let index = read_tree_to_index(tree_oid)?;
    for entry in &index {
        let oid = entry.0.clone();
        let path = Path::new(&entry.1);

        if let Some(parent) = path.parent() { 
            create_dir_all(parent)?;
        }
        let object_path = nag_dir.join("objects").join(oid);
        let obj_contents = read_file(&object_path.to_string_lossy());
        write_file(&obj_contents, &path)?;
    }

    write_index(&index)?;
    let head_path = nag_dir.join("HEAD");
    let new_head = format!("ref: refs/heads/{}", branch);
    write_file(&new_head.as_bytes().to_vec(), &head_path)?;

    Ok(())
}
