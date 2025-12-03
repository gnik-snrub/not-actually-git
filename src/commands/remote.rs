use crate::core::refs::{
    update_ref,
    get_ref_path,
};

pub fn add_remote(name: String, path: String) -> std::io::Result<()> {
    update_ref(&format!("refs/remotes/{}", name), &path)?;
    Ok(())
}

pub fn remove_remote(name: String) -> std::io::Result<()> {
    let ref_path = get_ref_path(&format!("refs/remotes/{}", name))?;
    if !ref_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Remote '{}' not found", name),
        ));
    }
    std::fs::remove_file(ref_path)?;
    Ok(())
}
