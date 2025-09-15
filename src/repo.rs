pub fn find_repo_root() -> Result<std::path::PathBuf, String> {
    let cwd = std::env::current_dir();
    match cwd {
        Err(_) => {
            return Err("Error: Could not find current directory".to_string())
        },
        Ok(mut curr) => {
            if curr.join(".nag").is_dir() {
                return Ok(curr.join(".nag"))
            }
            while let Some(parent) = curr.parent() {
                println!("{:?}", parent);
                if parent.join(".nag").is_dir() {
                    println!("{:?}", parent.join(".nag"));
                    return Ok(parent.join(".nag"))
                }
                curr = parent.to_path_buf();
            }
        }
    }
    return Err("Error: Could not find .nag project directory".to_string())
}
