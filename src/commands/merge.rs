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
use std::collections::HashSet;

pub fn merge(target_branch: String) -> std::io::Result<()> {
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

    let ancestor = find_ancestor_type(&object_dir, &oid, &target_commit_oid)?;

    match ancestor {
        Ancestor::Direct => {
            fast_forward(&branch, &target_commit_oid)?;
            println!("Fast-forwarded '{}' to '{}' (new commit: {})", branch, target_branch, target_commit_oid);
        },
        Ancestor::Shared(ancestor_oid) => {
            println!("3-way merge possible - not yet implemented");
        },
        Ancestor::NotFound => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Current branch is not an ancestor of the target branch"),
            ));
        },
    }

    Ok(())
}

fn fast_forward(branch: &String, target_commit_oid: &String) -> std::io::Result<()> {
    if status(false)?.len() > 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot fast-forward: working directory not clean"),
        ));
    }

    update_ref(&branch, &target_commit_oid)?;
    set_head_ref(&branch)?;

    let current_oid = read_ref(&branch)?;
    if &current_oid == target_commit_oid {
        println!("Already on up-to-date tree, no checkout needed");
        return Ok(());
    }
    checkout(branch.clone())?;

    Ok(())
}

#[derive(Debug, PartialEq)]
enum Ancestor {
    Direct,
    Shared(String),
    NotFound,
}

fn find_ancestor_type(object_dir: &Path, base_oid: &str, target_oid: &str) -> std::io::Result<Ancestor> {
    let mut target_parent_oids: HashSet<String> = HashSet::new();
    collect_all_ancestors(object_dir, target_oid, &mut target_parent_oids)?;

    if target_parent_oids.is_empty() {
        return Ok(Ancestor::NotFound)
    }

    if target_parent_oids.contains(&base_oid.to_string()) {
        return Ok(Ancestor::Direct)
    }

    return find_shared_ancestor(object_dir, base_oid, &target_parent_oids);
}

fn collect_all_ancestors(object_dir: &Path, target_oid: &str, parent_oids: &mut HashSet<String>) -> std::io::Result<()> {
    let new_parent_oids = collect_oids(object_dir, target_oid)?;

    if new_parent_oids.is_empty() {
        return Ok(())
    }

    for parent_oid in new_parent_oids.clone() {
        collect_all_ancestors(object_dir, &parent_oid, parent_oids)?;
    }

    parent_oids.extend(new_parent_oids);

    Ok(())
}

fn collect_oids(object_dir: &Path, oid: &str) -> std::io::Result<HashSet<String>> {
    let target_object_path = object_dir.join(oid);
    if !target_object_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit object {} not found", oid),
        ));
    }
    let commit_bytes = read_file(&target_object_path.to_string_lossy())?;
    let commit_str = String::from_utf8_lossy(&commit_bytes);
    let trimmed = commit_str.trim();

    let line_prefix = "parent ";
    let new_parent_oids: HashSet<String> = trimmed
        .lines()
        .filter(|line| line.starts_with(line_prefix))
        .map(|line| line[line_prefix.len()..].to_string())
        .collect::<HashSet<String>>();

    Ok(new_parent_oids)
}

fn find_shared_ancestor(object_dir: &Path, our_oid: &str, their_oids: &HashSet<String>) -> std::io::Result<Ancestor> {
    let mut our_parent_oids: HashSet<String> = collect_oids(object_dir, our_oid)?;

    if our_parent_oids.is_empty() {
        return Ok(Ancestor::NotFound)
    }

    for parent_oid in our_parent_oids {
        if their_oids.contains(&parent_oid) {
            return Ok(Ancestor::Shared(parent_oid))
        } else {
            return find_shared_ancestor(object_dir, &parent_oid, their_oids)
        }
    }

    Ok(Ancestor::NotFound)
}
