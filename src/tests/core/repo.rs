use std::fs;
use tempfile::TempDir;

use crate::core::repo::find_repo_root;

#[test]
fn finds_repo_in_current_dir() {
    let tmp = TempDir::new().unwrap();
    let nag = tmp.path();
    fs::create_dir_all(nag.join(".nag")).unwrap();

    std::env::set_current_dir(tmp.path()).unwrap();
    let root = find_repo_root().unwrap();
    assert_eq!(root, nag);
}

#[test]
fn walks_up_to_parent_repo() {
    let tmp = TempDir::new().unwrap();
    let nag = tmp.path();
    fs::create_dir_all(nag.join(".nag")).unwrap();

    let subdir = tmp.path().join("sub/dir");
    fs::create_dir_all(&subdir).unwrap();
    std::env::set_current_dir(&subdir).unwrap();

    let root = find_repo_root().unwrap();
    assert_eq!(root, nag);
}

#[test]
fn errors_if_no_repo() {
    let tmp = TempDir::new().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let root = find_repo_root();
    assert!(root.is_err());
}

#[test]
fn prefers_nearest_repo_in_nested_case() {
    let tmp = TempDir::new().unwrap();

    // parent repo
    let nag_parent = tmp.path();
    fs::create_dir_all(nag_parent.join(".nag")).unwrap();

    // child repo
    let child = tmp.path().join("child");
    fs::create_dir_all(&child).unwrap();
    let nag_child = &child;
    fs::create_dir_all(nag_child.join(".nag")).unwrap();

    std::env::set_current_dir(&child).unwrap();
    let root = find_repo_root().unwrap();

    assert_eq!(&root, nag_child);
}
