use tempfile::TempDir;
use std::fs;
use crate::commands::remote::{ add_remote, remove_remote };
use crate::core::repo::find_repo_root;
use crate::core::io::read_file;
use crate::commands::init::init;

fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

#[test]
fn add_remote_creates_ref_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    add_remote("origin".into(), "/some/path".into()).unwrap();

    let path = root.join(".nag/refs/remotes/origin");
    assert!(path.exists());

    let contents = read_file(&path.to_string_lossy()).unwrap();
    assert_eq!(String::from_utf8_lossy(&contents).trim(), "/some/path");
}

#[test]
fn add_remote_overwrites_existing() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    add_remote("x".into(), "/old/path".into()).unwrap();
    add_remote("x".into(), "/new/path".into()).unwrap();

    let path = root.join(".nag/refs/remotes/x");
    let contents = read_file(&path.to_string_lossy()).unwrap();
    assert_eq!(String::from_utf8_lossy(&contents).trim(), "/new/path");
}

#[test]
fn remove_remote_deletes_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    add_remote("r".into(), "/p".into()).unwrap();
    let path = root.join(".nag/refs/remotes/r");
    assert!(path.exists());

    remove_remote("r".into()).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_remote_errors_if_missing() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(&tmp);

    let res = remove_remote("nope".into());
    assert!(res.is_err());
}

#[test]
fn add_and_remove_remote_do_not_affect_branches() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    // create a branch to verify it's untouched
    let branch_path = root.join(".nag/refs/heads/main");
    assert!(branch_path.exists());

    add_remote("o".into(), "/p".into()).unwrap();
    remove_remote("o".into()).unwrap();

    assert!(branch_path.exists()); // ensure branch not deleted
}
