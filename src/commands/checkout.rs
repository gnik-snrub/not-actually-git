use crate::commands::status::status;
use crate::core::io::read_file;
use crate::core::repo::find_repo_root;

pub fn checkout(branch: String) -> std::io::Result<()> {
    if status(false)?.len() > 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("There are un-committed changes made. Please save your changes before checkout"),
        ));
    }

    let nag_dir = find_repo_root()?.join(".nag");
    let branch_path = nag_dir.join(format!("refs/heads/{}", branch));
    if !branch_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Branch '{}' not found", branch),
        ));
    }
    let branch_contents = read_file(&branch_path.to_string_lossy());
    let branch_str = String::from_utf8_lossy(&branch_contents);

    let commit_path = nag_dir.join("objects").join(branch_str.trim());
    if !commit_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit '{}' not found", branch),
        ));
    }
    let commit_contents = read_file(&commit_path.to_string_lossy());
    let commit_str = String::from_utf8_lossy(&commit_contents);

    let tree_line = commit_str.lines().next().unwrap();
    let tree_oid = tree_line.strip_prefix("tree ").unwrap().trim();
    let tree_path = nag_dir.join("objects").join(tree_oid.trim());
    if !tree_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit\'s tree '{}' not found", branch),
        ));
    }
    let tree_contents = read_file(&tree_path.to_string_lossy());
    let tree_str = String::from_utf8_lossy(&tree_contents);

    Ok(())
}
