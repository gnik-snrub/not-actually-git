use crate::core::hash::hash;
use crate::core::io::read_file;
use crate::core::index::read_index;
use crate::core::repo::find_repo_root;
use crate::core::tree::read_tree_to_index;
use crate::core::ignore::should_ignore;

use std::fs::read_dir;
use std::collections::{ HashMap, HashSet };
use std::path::Path;

#[derive(Eq, Hash, PartialEq)]
pub enum DiffType { Added, Modified, Deleted, Untracked, Staged, StagedDelete }

pub fn get_all_diffs() -> std::io::Result<HashMap<DiffType, Vec<String>>> {
    let mut diffs = HashMap::new();
    let working_diffs = diff_working_to_index()?;
    let index_diffs = diff_index_to_head()?;

    diffs.extend(working_diffs);
    diffs.extend(index_diffs);

    Ok(diffs)
}

pub fn diff_index_to_head() -> std::io::Result<HashMap<DiffType, Vec<String>>> {
    let mut tracker: HashMap<DiffType, Vec<String>> = HashMap::new();

    let index = read_index()?;
    let mut working: Vec<(String, String)> = vec![];
    let root = find_repo_root()?;

    walk(&root, &mut working, &root)?;

    let index_map: HashMap<String, String> = index.iter()
        .map(|(oid, p)| (p.clone(), oid.clone()))
        .collect();
    let wrk_paths: HashSet<_> = working.iter().map(|(_, p)| p.clone()).collect();

    let head_path = root.join(".nag").join("HEAD");
    let proj_head_contents = read_file(&head_path.to_string_lossy())?;
    let head_str = String::from_utf8_lossy(&proj_head_contents);

    let target = head_str.trim();
    let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
    let branch_path = root.join(".nag").join(branch_path_fragment);
    let branch_contents = read_file(&branch_path.to_string_lossy())?;
    let branch_oid = String::from_utf8_lossy(&branch_contents);

    let head_index_map = if branch_oid.trim().is_empty() {
        HashMap::new()
    } else {
        let commit_path = root.join(".nag").join("objects").join(format!("{}", branch_oid));
        let commit_contents = read_file(&commit_path.to_string_lossy())?;
        let commit_str = String::from_utf8_lossy(&commit_contents);

        let tree_line = commit_str
            .lines()
            .find(|line| line.starts_with("tree "))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing tree line"))?;
        let tree_oid = tree_line["tree ".len()..].trim();
        let head_index = read_tree_to_index(&tree_oid)?;

        head_index.into_iter()
            .map(|(oid, p)| (p.clone(), oid.clone()))
            .collect()
    };

    for index_entry in index {
        let index_entry_oid = index_entry.0;
        let index_entry_path = index_entry.1;
        if should_ignore(Path::new(&index_entry_path))? {
            continue;
        }
        if !wrk_paths.contains(&index_entry_path) {
            tracker.entry(DiffType::Deleted)
                .or_default()
                .push(index_entry_path.clone());
        }
        if let Some(head_entry_oid) = head_index_map.get(&index_entry_path) {
            if *head_entry_oid != index_entry_oid {
                tracker.entry(DiffType::Staged)
                    .or_default()
                    .push(index_entry_path);
            }
        } else {
            tracker.entry(DiffType::Added)
                .or_default()
                .push(index_entry_path);
        }
    }

    for (head_path, _head_oid) in head_index_map.iter() {
        if index_map.get(head_path).is_none() {
            tracker.entry(DiffType::StagedDelete)
                .or_default()
                .push(head_path.clone());
        }
    }

    Ok(tracker)
}

pub fn diff_working_to_index() -> std::io::Result<HashMap<DiffType, Vec<String>>> {
    let mut tracker: HashMap<DiffType, Vec<String>> = HashMap::new();

    let index = read_index()?;
    let mut working: Vec<(String, String)> = vec![];
    let root = find_repo_root()?;

    walk(&root, &mut working, &root)?;

    let index_map: HashMap<String, String> = index.iter()
        .map(|(oid, p)| (p.clone(), oid.clone()))
        .collect();

    for working_entry in working {
        let wrk_oid = working_entry.0;
        let wrk_path = working_entry.1;
        if should_ignore(Path::new(&wrk_path))? {
            continue;
        }
        if let Some(index_oid) = index_map.get(&wrk_path) {
            if &wrk_oid != index_oid {
                tracker.entry(DiffType::Modified)
                    .or_default()
                    .push(wrk_path);
            }
        } else {
            tracker.entry(DiffType::Untracked)
                .or_default()
                .push(wrk_path);
        }
    }
    Ok(tracker)
}

fn walk(path: &Path, working: &mut Vec<(String, String)>, root: &Path) -> std::io::Result<()> {
    if should_ignore(path)? {
        return Ok(());
    }
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

        let file = read_file(&abs_path.to_string_lossy())?;
        let blob = hash(&file);
        working.push((blob, rel_str));
    }
    Ok(())
}
