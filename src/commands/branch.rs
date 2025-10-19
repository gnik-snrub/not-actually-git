use crate::core::io::{ read_file, write_file };
use crate::core::repo::find_repo_root;

use std::fs::read_dir;
use std::path::Path;

pub fn branch(branch: String, source_oid: Option<String>) -> std::io::Result<()> {

    if branch_list(false)?.contains(&branch) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Branch '{}' already exists", branch),
        ));
    }

    let nag_head = find_repo_root()?.join(".nag");
    let refs_dir = nag_head.join("refs/heads");
    if refs_dir.join(&branch).exists() {
        println!("Branch {} already exists", branch);
        return Ok(());
    }

    if let Some(oid) = source_oid {
        if let Some(parent) = refs_dir.join(&branch).parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_file(&oid.as_bytes().to_vec(), &refs_dir.join(&branch))?;
        println!("Branch {} created at {}", branch, oid);
    } else {
        let proj_head = nag_head.join("HEAD");
        let proj_head_contents = read_file(&proj_head.to_string_lossy())?;
        let head_str = String::from_utf8_lossy(&proj_head_contents);

        let target = head_str.trim();
        let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
        let branch_path = nag_head.join(branch_path_fragment);
        let branch_contents = read_file(&branch_path.to_string_lossy())?;
        let oid = String::from_utf8_lossy(&branch_contents).trim().to_string();

        if let Some(parent) = refs_dir.join(&branch).parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_file(&oid.as_bytes().to_vec(), &refs_dir.join(&branch))?;

        println!("Branch {} created at {}", branch, oid);
    }
    Ok(())
}

pub fn branch_list(print: bool) -> std::io::Result<String> {
    let nag_head = find_repo_root()?.join(".nag");
    let refs_dir = nag_head.join("refs/heads");
    let proj_head = nag_head.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy())?;
    let head_str = String::from_utf8_lossy(&proj_head_contents);
    let trimmed = head_str.trim();
    let active_branch = trimmed.strip_prefix("ref: refs/heads/").unwrap_or(trimmed);

    let mut all_branches = Vec::new();
    collect_branches(&refs_dir, &mut all_branches, Some(String::new()))?;

    all_branches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let mut output = String::new();
    for entry in all_branches {
        if entry == active_branch {
            output.push('*');
        }
        output.push_str(&format!("{}\n", entry));
    }

    if print {
        println!("{output}");
    }

    Ok(output)
}

fn collect_branches(path: &Path, branches: &mut Vec<String>, prefix: Option<String>) -> std::io::Result<()> {
    if path.is_dir() {
        for child in read_dir(path)? {
            let dir = child.unwrap();
            let name = dir.file_name().to_string_lossy().to_string();

            let full_name = match &prefix {
                Some(pre) if !pre.is_empty() => format!("{}/{}", pre, name),
                _ => name.clone(),
            };

            if dir.path().is_dir() {
                collect_branches(&dir.path(), branches, Some(full_name))?;
            } else if dir.path().is_file() {
                branches.push(full_name);
            }
        }
    }
    Ok(())
}
