use crate::core::repo::find_repo_root;
use crate::core::io::read_file;

pub fn resolve_head() -> std::io::Result<(Option<String>, String)> {
    let nag_head = find_repo_root()?.join(".nag");
    let proj_head = nag_head.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy());
    let head_str = String::from_utf8_lossy(&proj_head_contents);
    let trimmed_head = head_str.trim();

    println!("head: {:?}", trimmed_head);

    if trimmed_head.starts_with("ref: ") {
        // HEAD is pointing to a branch
        let branch_path = nag_head.join(trimmed_head.strip_prefix("ref: ").unwrap_or(trimmed_head));
        let branch_path_str = branch_path.to_string_lossy().to_string();
        let branch_contents = read_file(&branch_path_str);
        let commit_oid = String::from_utf8_lossy(&branch_contents);
        let trimmed_oid = commit_oid.trim();

        let branch_name = branch_path.file_name().unwrap();

        return Ok((Some(branch_name.to_string_lossy().to_string()), trimmed_oid.to_string()))
    } else {
        // HEAD is detached, and directly contains a commit oid
        return Ok((None, trimmed_head.to_string()))
    }
}

