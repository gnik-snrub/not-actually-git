use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

use crate::core::tree::{write_tree, write_tree_from_index};
use crate::core::index::IndexEntry;
use crate::core::index::EntryType;
use crate::core::repo::find_repo_root;
use crate::core::hash::hash;
use crate::core::io::{read_file, write_object};

fn init_repo(tmp: &TempDir) {
    let nag_root = tmp.path().join(".nag");
    let objects = nag_root.join("objects");
    fs::create_dir_all(objects).unwrap();

    std::env::set_current_dir(tmp.path()).unwrap();
}

#[test]
fn write_tree_with_single_file() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("file.txt");
    fs::write(&file_path, b"hello").unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();

    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    assert!(tree_path.exists());

    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap()).unwrap()).unwrap();
    assert!(tree_content.contains("100644\tfile.txt"));
}

#[test]
fn write_tree_with_nested_dir() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let nested = tmp.path().join("subdir");
    fs::create_dir(&nested).unwrap();
    let file_path = nested.join("note.txt");
    fs::write(&file_path, b"nested").unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap()).unwrap()).unwrap();

    assert!(tree_content.contains("040000\tsubdir"));
}

#[test]
fn write_tree_sets_exec_permission() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("run.sh");
    fs::write(&file_path, b"#!/bin/sh\necho hi").unwrap();
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file_path, perms).unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap()).unwrap()).unwrap();

    assert!(tree_content.contains("100755\trun.sh"));
}

#[test]
fn write_tree_skips_nag_dir() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let hidden = tmp.path().join(".nag/hidden.txt");
    fs::write(&hidden, b"secret").unwrap();

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap()).unwrap()).unwrap();

    assert!(!tree_content.contains("hidden.txt"));
}

#[test]
fn write_tree_empty_repo() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let tree_hash = write_tree(&tmp.path().to_path_buf()).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_hash);
    let tree_content = String::from_utf8(read_file(tree_path.to_str().unwrap()).unwrap()).unwrap();

    assert!(tree_content.is_empty());
}

#[test]
fn write_tree_from_index_single_file() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let blob_bytes = b"hello".to_vec();
    let oid = hash(&blob_bytes);
    write_object(&blob_bytes, &oid).unwrap();

    let entries = vec![
        IndexEntry {
            entry_type: EntryType::C,
            mode: "100644".to_string(),
            path: "file.txt".to_string(),
            oids: vec![oid.clone()],
        }
    ];

    let tree_oid = write_tree_from_index(&entries).unwrap();

    let tree_path = tmp.path().join(".nag/objects").join(&tree_oid);
    assert!(tree_path.exists());

    let tree_data = fs::read_to_string(tree_path).unwrap();
    assert!(tree_data.contains("file.txt"));
    assert!(tree_data.contains(&oid));
}

#[test]
fn write_tree_from_index_nested_directories() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let blob_a = b"fn a() {}".to_vec();
    let blob_b = b"fn b() {}".to_vec();

    let oid_a = hash(&blob_a);
    let oid_b = hash(&blob_b);

    write_object(&blob_a, &oid_a).unwrap();
    write_object(&blob_b, &oid_b).unwrap();

    let entries = vec![
        IndexEntry {
            entry_type: EntryType::C,
            mode: "100644".to_string(),
            path: "src/a.rs".to_string(),
            oids: vec![oid_a.clone()],
        },
        IndexEntry {
            entry_type: EntryType::C,
            mode: "100644".to_string(),
            path: "src/b.rs".to_string(),
            oids: vec![oid_b.clone()],
        }
    ];

    let tree_oid = write_tree_from_index(&entries).unwrap();
    let tree_path = tmp.path().join(".nag/objects").join(&tree_oid);
    let tree_data = fs::read_to_string(tree_path).unwrap();

    assert!(tree_data.contains("src"));
    assert!(tree_data.contains("040000"));
}

#[test]
fn write_tree_from_index_multiple_directories() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let blob_a = b"A".to_vec();
    let blob_b = b"B".to_vec();
    let oid_a = hash(&blob_a);
    let oid_b = hash(&blob_b);

    write_object(&blob_a, &oid_a).unwrap();
    write_object(&blob_b, &oid_b).unwrap();

    let entries = vec![
        IndexEntry {
            entry_type: EntryType::C,
            mode: "100644".to_string(),
            path: "dir1/file_a.txt".to_string(),
            oids: vec![oid_a.clone()],
        },
        IndexEntry {
            entry_type: EntryType::C,
            mode: "100644".to_string(),
            path: "dir2/file_b.txt".to_string(),
            oids: vec![oid_b.clone()],
        }
    ];

    let tree_oid = write_tree_from_index(&entries).unwrap();
    let tree_data = fs::read_to_string(tmp.path().join(".nag/objects").join(&tree_oid)).unwrap();

    assert!(tree_data.contains("dir1"));
    assert!(tree_data.contains("dir2"));
}

#[test]
fn write_tree_from_index_empty_index_creates_empty_tree() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let entries: Vec<IndexEntry> = vec![];

    let tree_oid = write_tree_from_index(&entries).unwrap();
    let tree_data = fs::read_to_string(tmp.path().join(".nag/objects").join(&tree_oid)).unwrap();

    assert!(tree_data.is_empty() || !tree_data.contains('\n'));
}

#[test]
fn write_tree_from_index_missing_blob_errors() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let fake_oid = "deadbeef".repeat(8);

    let entries = vec![
        IndexEntry {
            entry_type: EntryType::C,
            mode: "100644".to_string(),
            path: "ghost.txt".to_string(),
            oids: vec![fake_oid.clone()],
        }
    ];

    let result = write_tree_from_index(&entries);

    assert!(result.is_err(), "should error on missing blob object");
}
