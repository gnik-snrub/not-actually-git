use crate::core::repo::find_repo_root;
use crate::core::io::read_file;
use crate::core::tree::read_tree_to_index;

use std::collections::HashMap;

pub fn restore(restore_path: String) -> std::io::Result<()> {
    let root = find_repo_root()?;
    let nag_dir = root.join(".nag");

    let proj_head = nag_dir.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy());
    let head_str = String::from_utf8_lossy(&proj_head_contents);

    let target = head_str.trim();
    let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
    let branch_path = nag_dir.join(branch_path_fragment);
    let branch_contents = read_file(&branch_path.to_string_lossy());
    let branch_str = String::from_utf8_lossy(&branch_contents).trim().to_string();

    let commit_path = nag_dir.join("objects").join(branch_str.trim());
    let commit_contents = read_file(&commit_path.to_string_lossy());
    let commit_str = String::from_utf8_lossy(&commit_contents);

    let tree_line = commit_str.lines().next().unwrap();
    let tree_oid = tree_line.strip_prefix("tree ").unwrap().trim();

    let tree_map: HashMap<String, String> = read_tree_to_index(&tree_oid)?
        .into_iter()
        .map(|(oid, path)| {
            return (path, oid)
        })
        .collect();

    println!("tree_map: {:?}", tree_map);

    Ok(())
}

fn walk(path: String) -> std::io::Result<()> {

}
