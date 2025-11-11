use crate::core::refs::{
    resolve_head,
    read_ref,
    update_ref,
    set_head_ref,
};
use crate::core::io::{ read_file, write_file };
use crate::core::repo::find_repo_root;
use crate::commands::checkout::checkout;
use crate::commands::status::status;
use crate::core::tree::read_tree_to_index;

use std::path::Path;
use std::collections::{ HashMap, HashSet };

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
            three_way_merge(&oid, &target_commit_oid, &ancestor_oid)?;
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

fn three_way_merge(base_oid: &str, target_oid: &str, ancestor_oid: &str) -> std::io::Result<()> {
    let base_index = read_tree_to_index(base_oid)?.into_iter().map(|(oid, path)| (path, oid)).collect::<HashMap<String, String>>();
    let target_index = read_tree_to_index(target_oid)?.into_iter().map(|(oid, path)| (path, oid)).collect::<HashMap<String, String>>();
    let ancestor_index = read_tree_to_index(ancestor_oid)?.into_iter().map(|(oid, path)| (path, oid)).collect::<HashMap<String, String>>();

    let mut map: HashMap<String, (Option<String>, Option<String>, Option<String>)> = HashMap::new();
    for (path, oid) in ancestor_index {
        map.entry(path).or_insert((None,None,None)).0 = Some(oid);
    }

    for (path, oid) in base_index {
        map.entry(path).or_insert((None,None,None)).1 = Some(oid);
    }

    for (path, oid) in target_index {
        map.entry(path).or_insert((None,None,None)).2 = Some(oid);
    }

    let mut final_index: Vec<(String, Vec<String>)> = Vec::new();

    for (path, (base_oid, target_oid, ancestor_oid)) in map {
        match (ancestor_oid, base_oid, target_oid) {
            (Some(a), Some(b), Some(t)) => {
                if a == b && a == t {
                    final_index.push((path, vec![a]));
                } else if a == b && a != t {
                    final_index.push((path, vec![t]));
                } else if a != b && a == t {
                    final_index.push((path, vec![b]));
                } else if a != b && b == t {
                    final_index.push((path, vec![b]));
                } else if a != b && a != t && b != t {
                    // Conflict, so keep both
                    final_index.push((path, vec![b, t]));
                }
            },
            (Some(_), Some(b), None) | (None, Some(b), None) => {
                final_index.push((path, vec![b]));
            },
            (Some(_), None, Some(t)) | (None, None, Some(t)) => {
                final_index.push((path, vec![t]));
            },
            (None, Some(b), Some(t)) => {
                if b == t {
                    // Somehow, both branches independently made the same change
                    final_index.push((path, vec![b]));
                } else {
                    // Conflict, so keep both
                    final_index.push((path, vec![b, t]));
                }
            },
            _ => {
                // Other matches don't matter, so nothing is kept
            }
        }
    }

    for (path, oids) in final_index {
        if oids.len() > 1 {
            build_conflict_file(&oids[0], &oids[1], &path)?;
        } else {
            let file_path = Path::new(&path);
            write_file(&oids[0].as_bytes().to_vec(), &file_path)?;
        }
    }

    println!("3-way merge possible - not yet implemented");
    Ok(())
}

fn build_conflict_file(base_oid: &str, target_oid: &str, conflict_file: &str) -> std::io::Result<()> {
    let object_dir = find_repo_root()?.join(".nag").join("objects");
    let base_object_path = object_dir.join(base_oid);
    let target_object_path = object_dir.join(target_oid);

    let base_object = read_file(&base_object_path.to_string_lossy())?;
    let target_object = read_file(&target_object_path.to_string_lossy())?;
    let base_object_str = String::from_utf8_lossy(&base_object);
    let target_object_str = String::from_utf8_lossy(&target_object);

    let mut str_buf = String::new();
    let header = format!("<<< Base <<<\n");
    let mid_line = format!("==============\n");
    let footer = format!(">>> Target >>>\n");

    str_buf.push_str(&header);
    str_buf.push_str(&base_object_str);
    str_buf.push_str(&mid_line);
    str_buf.push_str(&target_object_str);
    str_buf.push_str(&footer);

    let conflict_path = Path::new(conflict_file);

    write_file(&str_buf.as_bytes().to_vec(), &conflict_path)?;

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
    let our_parent_oids: HashSet<String> = collect_oids(object_dir, our_oid)?;

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
