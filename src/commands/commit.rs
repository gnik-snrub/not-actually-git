use crate::core::index::{ read_index, write_index };
use crate::core::tree::{ write_tree_from_index, read_tree_to_index };
use crate::core::repo::find_repo_root;
use crate::core::io::{ read_file, write_object, write_file };
use crate::core::hash::hash;

pub fn commit(message: String) -> std::io::Result<()> {
    let mut commit_str_buf = String::new();

    let index = read_index()?;
    let tree = write_tree_from_index(&index)?;
    commit_str_buf.push_str(&format!("tree {}\n", tree.trim()));

    let nag_head = find_repo_root()?.join(".nag");
    let proj_head = nag_head.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy());
    let head_str = String::from_utf8_lossy(&proj_head_contents);

    let target = head_str.trim();
    let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
    let branch_path = nag_head.join(branch_path_fragment);
    let branch_contents = read_file(&branch_path.to_string_lossy());
    let branch_str = String::from_utf8_lossy(&branch_contents);

    if !branch_str.trim().is_empty() {
        commit_str_buf.push_str(&format!("parent {}\n", branch_str.trim()));
    }

    // TODO Build author / user config and add in an author line

    commit_str_buf.push_str(&format!("\n{}\n", message.trim()));

    let buffer_bytes = commit_str_buf.into_bytes();
    let commit_hash = hash(&buffer_bytes);
    write_object(&buffer_bytes, &commit_hash)?;

    write_file(&commit_hash.into_bytes(), &branch_path)?;

    let committed_index = read_tree_to_index(&tree)?;
    write_index(&committed_index)?;

    Ok(())
}
