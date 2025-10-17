use crate::core::io::{ read_file, write_file };
use crate::core::repo::find_repo_root;

use std::fs::read_dir;

pub fn branch(branch: String, source_oid: Option<String>) -> std::io::Result<()> {

    if branch_list(false)?.contains(&branch) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Branch '{}' already exists", branch),
        ));
    }

    let refs_dir = nag_head.join("refs/heads");
    if refs_dir.join(&branch).exists() {
        println!("Branch {} already exists", branch);
        return Ok(());
    }

pub fn branch_list(print: bool) -> std::io::Result<String> {
    let nag_head = find_repo_root()?.join(".nag");
    let refs_dir = nag_head.join("refs/heads");
    let proj_head = nag_head.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy());
    let head_str = String::from_utf8_lossy(&proj_head_contents);
    let trimmed = head_str.trim();
    let active_branch = trimmed.strip_prefix("ref: refs/heads/").unwrap_or(trimmed);

    let mut output = String::new();
    let mut branches: Vec<String> = read_dir(refs_dir)?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();

    branches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    for entry in branches {
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
