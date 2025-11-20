use crate::core::refs::{
    resolve_head,
    read_ref,
    update_ref,
    set_head_ref,
};
use crate::core::io::{ read_file, write_file };
use crate::core::repo::find_repo_root;
use crate::core::tree::read_tree_to_index;
use crate::core::index::{ write_index, IndexEntry, EntryType };
use crate::commands::checkout::checkout;
use crate::commands::status::status;

use std::path::Path;
use std::collections::{ HashMap, HashSet };

pub fn merge(target_branch: String) -> std::io::Result<()> {
    if status(false)?.len() > 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot merge: working directory not clean"),
        ));
    }

    let head = resolve_head()?;

    let (Some(branch), oid) = head else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Cannot fast-forward detached HEAD"),
        ));
    };

    let target_commit_oid = read_ref(&target_branch)?;

    if oid == target_commit_oid {
        println!("Already up-to-date");
        return Ok(());
    }

    let nag_dir = find_repo_root()?.join(".nag");
    let object_dir = nag_dir.join("objects");

    let ancestor = find_ancestor_type(&object_dir, &oid, &target_commit_oid)?;

    match ancestor {
        Ancestor::Direct => {
            fast_forward(&branch, &target_commit_oid)?;
            println!("Fast-forwarded '{}' to '{}' (new commit: {})", branch, target_branch, target_commit_oid);
        },
        Ancestor::DirectReverse => {
            println!("Already up-to-date");
        },
        Ancestor::Shared(ancestor_oid) => {
            let output = three_way_merge(&oid, &target_commit_oid, &ancestor_oid)?;
            println!("{}", output);
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
    update_ref(&branch, &target_commit_oid)?;
    set_head_ref(&branch)?;

    Ok(())
}

fn extract_tree_oid(commit_str: &str) -> std::io::Result<String> {
    let nag_dir = find_repo_root()?.join(".nag");
    let tree_oid = commit_str.lines().next().unwrap();
    let tree_path = nag_dir.join("objects").join(tree_oid.trim());
    if !tree_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit\'s tree '{}' not found", tree_oid),
        ));
    }
    let tree_contents = read_file(&tree_path.to_string_lossy())?;
    let tree_str = String::from_utf8_lossy(&tree_contents);
    let first_line = tree_str.lines().next().unwrap();
    let tree_oid = first_line.strip_prefix("tree ").unwrap().trim();
    Ok(tree_oid.trim().to_string())
}

fn three_way_merge(base_oid: &str, target_oid: &str, ancestor_oid: &str) -> std::io::Result<String> {
    let base_tree = extract_tree_oid(base_oid)?;
    let target_tree = extract_tree_oid(target_oid)?;
    let ancestor_tree = extract_tree_oid(ancestor_oid)?;

    let base_index = read_tree_to_index(&base_tree)?.into_iter().map(|entry| (entry.path.clone(), entry)).collect::<HashMap<String, IndexEntry>>();
    let target_index = read_tree_to_index(&target_tree)?.into_iter().map(|entry| (entry.path.clone(), entry)).collect::<HashMap<String, IndexEntry>>();
    let ancestor_index = read_tree_to_index(&ancestor_tree)?.into_iter().map(|entry| (entry.path.clone(), entry)).collect::<HashMap<String, IndexEntry>>();

    let mut map: HashMap<String, (Option<IndexEntry>, Option<IndexEntry>, Option<IndexEntry>)> = HashMap::new();
    for (path, entry) in ancestor_index {
        map.entry(path).or_insert((None,None,None)).0 = Some(entry);
    }

    for (path, entry) in base_index {
        map.entry(path).or_insert((None,None,None)).1 = Some(entry);
    }

    for (path, entry) in target_index {
        map.entry(path).or_insert((None,None,None)).2 = Some(entry);
    }

    let mut final_index: Vec<IndexEntry> = Vec::new();

    for (_path, (a_entry, b_entry, t_entry)) in map {
        let is_dir =
            a_entry.clone().map(|e| e.mode == "040000").unwrap_or(false) ||
            b_entry.clone().map(|e| e.mode == "040000").unwrap_or(false) ||
            t_entry.clone().map(|e| e.mode == "040000").unwrap_or(false);

        if is_dir {
            continue; // directories never get index entries
        }
        match (a_entry, b_entry, t_entry) {
            (Some(a), Some(b), Some(t)) => {
                let aoid = a.oids[0].clone();
                let boid = b.oids[0].clone();
                let toid = t.oids[0].clone();
                if aoid == boid && aoid == toid {
                    final_index.push(quick_entry(&a, &vec![aoid]));
                } else if aoid == boid && aoid != toid {
                    final_index.push(quick_entry(&b, &vec![toid]));
                } else if aoid != boid && aoid == toid {
                    final_index.push(quick_entry(&b, &vec![boid]));
                } else if aoid != boid && boid == toid {
                    final_index.push(quick_entry(&b, &vec![boid]));
                } else if aoid != boid && aoid != toid && boid != toid {
                    // Conflict, so keep both
                    final_index.push(quick_entry(&b, &vec![boid, toid]));
                }
            },
            (Some(a), Some(b), None) => {
                let boid = b.oids[0].clone();
                let aoid = a.oids[0].clone();
                if aoid == boid {
                    // Clean delete
                } else {
                    final_index.push(quick_entry(&b, &vec![boid, "empty".to_string()]));
                }
            },
            (None, Some(b), None) => {
                let boid = b.oids[0].clone();
                final_index.push(quick_entry(&b, &vec![boid]));
            },
            (Some(a), None, Some(t)) => {
                let toid = t.oids[0].clone();
                let aoid = a.oids[0].clone();
                if aoid == toid {
                    // Clean delete
                } else {
                    final_index.push(quick_entry(&t, &vec![toid, "empty".to_string()]));
                }
            },
            (None, None, Some(t)) => {
                let toid = t.oids[0].clone();
                final_index.push(quick_entry(&t, &vec![toid]));
            },
            (None, Some(b), Some(t)) => {
                let boid = b.oids[0].clone();
                let toid = t.oids[0].clone();
                if boid == toid {
                    // Somehow, both branches independently made the same change
                    final_index.push(quick_entry(&b, &vec![boid]));
                } else {
                    // Conflict, so keep both
                    final_index.push(quick_entry(&b, &vec![boid, toid]));
                }
            },
            _ => {
                // Other matches don't matter, so nothing is kept
            }
        }
    }

    let repo_root = find_repo_root()?;
    for entry in &final_index {
        if entry.oids.len() > 1 {
            build_conflict_file(&entry.oids[0], &entry.oids[1], &entry.path)?;
        } else {
            let full_path = repo_root.join(&entry.path);
            let obj_path = repo_root.join(".nag/objects").join(&entry.oids[0]);
            let contents = read_file(obj_path.to_str().unwrap())?;

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            write_file(&contents, &full_path)?;
        }
    }

    write_index(&final_index)?;

    let mut summary_buf = String::new();
    summary_buf.push_str("Merge results:\n");
    for entry in &final_index {
        if entry.entry_type == EntryType::C {
            summary_buf.push_str(&format!("\tclean: {}\n", entry.path));
        } else {
            summary_buf.push_str(&format!("\tconflict: {}\n", entry.path));
        }
    }
    let has_conflicts = final_index.iter().any(|e| e.entry_type == EntryType::X);

    if has_conflicts {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Merge conflicts"))
    } else {
        Ok(summary_buf)
    }
}

fn quick_entry(existing_entry: &IndexEntry, oids: &Vec<String>) -> IndexEntry {
    IndexEntry {
        entry_type: if oids.len() > 1 { EntryType::X } else { EntryType::C },
        path: existing_entry.path.clone(),
        mode: existing_entry.mode.clone(),
        oids: oids.clone(),
    }
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
    DirectReverse,
    Shared(String),
    NotFound,
}

fn find_ancestor_type(object_dir: &Path, base_oid: &str, target_oid: &str) -> std::io::Result<Ancestor> {
    let mut target_parent_oids = HashSet::new();
    collect_all_ancestors(object_dir, target_oid, &mut target_parent_oids)?;

    let mut base_parent_oids = HashSet::new();
    collect_all_ancestors(object_dir, base_oid, &mut base_parent_oids)?;

    // Shared
    if let Some(shared) = base_parent_oids.intersection(&target_parent_oids).next() {
        return Ok(Ancestor::Shared(shared.clone()));
    }

    // Direct
    if target_parent_oids.contains(base_oid) {
        return Ok(Ancestor::Direct);
    }

    // DirectReverse
    if base_parent_oids.contains(target_oid) {
        return Ok(Ancestor::DirectReverse);
    }

    Ok(Ancestor::NotFound)
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
