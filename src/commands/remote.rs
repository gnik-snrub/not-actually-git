use crate::core::refs::{
    update_ref,
    get_ref_path,
};

use std::path::{ Path, PathBuf };

pub fn add_remote(name: String, path: String) -> std::io::Result<()> {
    let nag_path = get_remote_nag_dir(&path)?;
    if !nag_path.exists()
        && nag_path.join("refs/heads").exists()
        && nag_path.join("objects").exists()
        && nag_path.join("HEAD").exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Not a NAG repository",
        ));
    }
    update_ref(&format!("refs/remotes/{}", name), &path)?;
    return Ok(())
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

pub fn fetch_remote(name: String) -> std::io::Result<()> {

    Ok(())
}

fn get_remote_nag_dir(path: &String) -> std::io::Result<PathBuf> {
    let nag_path = Path::new(&path).join(".nag");
    if nag_path.is_dir() {
        return Ok(nag_path)
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Not a NAG repository",
    ))
}
