pub fn find_repo_root() -> std::io::Result<std::path::PathBuf> {
    let mut cwd = std::env::current_dir()?;
    if cwd.join(".nag").is_dir() {
        return Ok(cwd.to_path_buf())
    }
    while let Some(parent) = cwd.parent() {
        if parent.join(".nag").is_dir() {
            return Ok(parent.to_path_buf())
        }
        cwd = parent.to_path_buf();
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Not a NAG repository",
    ))
}
