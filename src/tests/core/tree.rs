use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

use crate::core::tree::write_tree;
use crate::core::io::read_file;

fn init_repo(tmp: &TempDir) {
    let nag_root = tmp.path().join(".nag");
    let objects = nag_root.join("objects");
    fs::create_dir_all(objects).unwrap();

    // important: pretend we’re “inside” the repo root
    std::env::set_current_dir(tmp.path()).unwrap();
}

#[test]
fn write_tree_with_single_file() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    // write file
    let file_path = tmp.path().join("file.txt");
    fs::write(&file_path, b"hello").unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();

    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    assert!(tree_path.exists());

    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap())).unwrap();
    assert!(tree_content.contains("100644\tfile.txt"));
}

#[test]
fn write_tree_with_nested_dir() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    // write nested file
    let nested = tmp.path().join("subdir");
    fs::create_dir(&nested).unwrap();
    let file_path = nested.join("note.txt");
    fs::write(&file_path, b"nested").unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap())).unwrap();

    assert!(tree_content.contains("040000\tsubdir")); // directory entry
}

#[test]
fn write_tree_sets_exec_permission() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    // create exec file
    let file_path = tmp.path().join("run.sh");
    fs::write(&file_path, b"#!/bin/sh\necho hi").unwrap();
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file_path, perms).unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap())).unwrap();

    assert!(tree_content.contains("100755\trun.sh"));
}

#[test]
fn write_tree_skips_nag_dir() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    // put junk file inside .nag
    let hidden = tmp.path().join(".nag/hidden.txt");
    fs::write(&hidden, b"secret").unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap())).unwrap();

    assert!(!tree_content.contains("hidden.txt"));
}

#[test]
fn write_tree_empty_repo() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap())).unwrap();

    assert!(tree_content.is_empty());
}
