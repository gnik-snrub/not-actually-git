use crate::core::io::{ read_file, write_object, write_file };
use crate::core::repo::{ find_repo_root };
use crate::core::tree::{ read_tree_to_index };

use std::path::{ Path, PathBuf };
use std::fs::read_dir;

pub fn add_remote(name: String, path: String) -> std::io::Result<()> {
    let nag_path = get_remote_nag_dir(&path)?;
    if !nag_path.exists()
        && nag_path.join("refs/heads").exists()
        && nag_path.join("objects").exists()
        && nag_path.join("HEAD").exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Not a NAG repository",
        ));
    }
    update_remote(&name, &path)?;
    return Ok(())
}

pub fn remove_remote(name: String) -> std::io::Result<()> {
    let nag_dir = find_repo_root()?.join(".nag");
    let remote_path = nag_dir.join("remotes").join(&name);
    if !remote_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Remote '{}' not found", name),
        ));
    }
    std::fs::remove_file(remote_path)?;
    Ok(())
}

pub fn fetch_remote(remote_name: String) -> std::io::Result<()> {
    let rem_path = find_repo_root()?.join(".nag/remotes").join(&remote_name);
    let rem_contents = read_file(&rem_path.to_string_lossy())?;
    let rem_str = String::from_utf8_lossy(&rem_contents);
    let remote = rem_str.trim().to_string();
    let remote_nag_dir = get_remote_nag_dir(&remote)?;

    let remote_heads_dir = remote_nag_dir.join("refs/heads");
    for entry in read_dir(remote_heads_dir)? {
        let entry = entry?;
        let branch_name = entry.file_name().to_string_lossy().to_string();
        let commit_oid_bytes = read_file(&entry.path().to_string_lossy().to_string())?;
        let commit_oid = String::from_utf8_lossy(&commit_oid_bytes).trim().to_string();

        let remote_objects_dir = remote_nag_dir.join("objects");
        let root_commit_obj_path = remote_objects_dir.join(&commit_oid);
        let root_commit_obj_bytes = read_file(&root_commit_obj_path.to_string_lossy().to_string())?;
        let root_commit_obj = String::from_utf8_lossy(&root_commit_obj_bytes).to_string();
        let root_tree_oid_line = root_commit_obj.lines().next().unwrap();
        let root_tree_oid = root_tree_oid_line.strip_prefix("tree ").unwrap_or(root_tree_oid_line);

        let local_root = find_repo_root()?;
        let local_objects_dir = local_root.join(".nag").join("objects");

        let mut queue: Vec<String> = vec![root_tree_oid.to_string()];
        while !queue.is_empty() {
            let oid = queue.pop().unwrap();
            if local_objects_dir.join(&oid).exists() {
                continue;
            }
            let remote_obj_path = remote_objects_dir.join(&oid);
            let remote_obj = read_file(&remote_obj_path.to_string_lossy().to_string())?;
            write_object(&remote_obj, &oid)?;

            let possible_index = read_tree_to_index(&oid);
            if let Ok(index) = possible_index {
                for entry in index.iter() {
                    queue.push(entry.oids[0].clone());
                }
            }
        }

        // Only write the head commit if it doesn't already exist
        if !local_objects_dir.join(&commit_oid).exists() {
            write_object(&root_commit_obj_bytes, &commit_oid)?;
        }

        let root_commit_parent = root_commit_obj
            .lines()
            .find(|l| l.starts_with("parent "))
            .and_then(|line| line.strip_prefix("parent "))
            .map(|s| s.trim().to_string());

        // Then walk the parents if they exist
        if let Some(parent) = root_commit_parent {
            let mut commit_cursor = parent;
            loop {
                let commit_path = remote_objects_dir.join(&commit_cursor);
                let commit_bytes = read_file(&commit_path.to_string_lossy())?;
                write_object(&commit_bytes, &commit_cursor)?;
                let commit_str = String::from_utf8_lossy(&commit_bytes);
                if let Some(parent_line) = commit_str.lines().find(|l| l.starts_with("parent ")) {
                    commit_cursor = parent_line.strip_prefix("parent ").unwrap().trim().to_string();
                } else {
                    break;
                }
                if local_objects_dir.join(&commit_cursor).exists() {
                    break;
                }
            }
        }

        let local_remote_ref_dir = local_root.join(".nag/refs/remotes").join(&remote_name);
        let tracking_ref_path = local_remote_ref_dir.join(&branch_name);

        // Only write if the ref doesn't exist or has a different value
        let should_write = if tracking_ref_path.exists() {
            let existing = read_file(&tracking_ref_path.to_string_lossy()).ok();
            existing.as_ref().map(|v| v.as_slice()) != Some(commit_oid.as_bytes())
        } else {
            true
        };

        if should_write {
            write_file(&commit_oid.as_bytes().to_vec(), &tracking_ref_path)?;
        }
    }

    Ok(())
}

fn get_remote_nag_dir(path: &String) -> std::io::Result<PathBuf> {
    let nag_path = Path::new(&path).join(".nag");
    if nag_path.is_dir() {
        return Ok(nag_path)
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Not a NAG repository",
    ))
}

pub fn update_remote(name: &str, url: &str) -> std::io::Result<()> {
    let full_path = find_repo_root()?.join(".nag/remotes").join(name);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    write_file(&url.as_bytes().to_vec(), &full_path)?;

    Ok(())
}
