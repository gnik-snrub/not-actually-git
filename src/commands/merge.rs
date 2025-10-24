use crate::core::refs::{
    resolve_head,
    read_ref,
    update_ref,
    set_head_ref,
};
use crate::core::io::read_file;
use crate::core::repo::find_repo_root;
use crate::commands::checkout::checkout;
use crate::commands::status::status;

use std::path::Path;

pub fn ff_merge(target_branch: String) -> std::io::Result<()> {
    let head = resolve_head()?;

    let (Some(branch), oid) = head else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Cannot fast-forward detached HEAD"),
        ));
    };

    let target_commit_oid = read_ref(&target_branch)?;

    if oid == target_commit_oid {
        println!("Already up to date");
        return Ok(())
    }

    let nag_dir = find_repo_root()?.join(".nag");
    let object_dir = nag_dir.join("objects");

    let ancestor = is_ancestor(&object_dir, &oid, &target_commit_oid)?;

    if ancestor {
        if status(false)?.len() > 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Cannot fast-forward: working directory not clean"),
            ));
        }

        update_ref(&branch, &target_commit_oid)?;
        set_head_ref(&branch)?;

        let current_oid = read_ref(&branch)?;
        if current_oid == target_commit_oid {
            println!("Already on up-to-date tree, no checkout needed");
            return Ok(());
        }
        checkout(branch.clone())?;

        println!("Fast-forwarded '{}' to '{}' (new commit: {})", branch, target_branch, target_commit_oid);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Current branch is not an ancestor of the target branch"),
        ));
    }

    Ok(())
}

fn is_ancestor(object_dir: &Path, base_oid: &str, target_oid: &str) -> std::io::Result<bool> {
    let target_object_path = object_dir.join(target_oid);
    if !target_object_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit object {} not found", target_oid),
        ));
    }
    let commit_bytes = read_file(&target_object_path.to_string_lossy())?;
    let commit_str = String::from_utf8_lossy(&commit_bytes);
    let trimmed = commit_str.trim();

    let line_prefix = "parent ";
    let parent_oids: Vec<String> = trimmed
        .lines()
        .filter(|line| line.starts_with(line_prefix))
        .map(|line| line[line_prefix.len()..].to_string())
        .collect::<Vec<String>>();

    if parent_oids.is_empty() {
        return Ok(false)
    }

    if parent_oids.contains(&base_oid.to_string()) {
        return Ok(true)
    } 

    for parent_oid in &parent_oids {
        if is_ancestor(object_dir, base_oid, &parent_oid)? {
            return Ok(true)
        }
    }
    Ok(false)
}
