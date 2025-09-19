use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a fake NAG repo in `tmp` and set cwd to it.
/// Returns the `.nag/objects` path.
pub fn setup_nag_repo(tmp: &TempDir) -> PathBuf {
    let nag = tmp.path().join(".nag");
    let objects = nag.join("objects");

    fs::create_dir_all(&objects).unwrap();
    fs::write(nag.join("HEAD"), b"ref: refs/heads/main").unwrap();
    fs::create_dir_all(nag.join("refs").join("heads")).unwrap();
    fs::write(nag.join("refs").join("heads").join("main"), b"").unwrap();

    std::env::set_current_dir(tmp.path()).unwrap();

    objects
}
