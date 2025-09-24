use std::fs;
use std::path::Path;
use tempfile::TempDir;

use crate::core::index::read_index;
use crate::commands::add::add;

// Helper: initialize fake repo with .nag structure and cd into it
fn init_repo(tmp: &TempDir) -> std::path::PathBuf {
    let nag_root = tmp.path().join(".nag");
    let objects = nag_root.join("objects");
    fs::create_dir_all(&objects).unwrap();

    // important: pretend we’re “inside” the repo root
    std::env::set_current_dir(tmp.path()).unwrap();

    nag_root
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn add_single_file_creates_blob_and_index_entry() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("hello.txt");
    write_file(&file_path, "hello world");

    add(&file_path).unwrap();

    // Index should contain the file (as relative path)
    let entries = read_index().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1, "hello.txt");

    // Blob should exist on disk
    let oid = &entries[0].0;
    let blob_path = tmp.path().join(".nag/objects").join(oid);
    assert!(blob_path.exists());
}

#[test]
fn add_directory_recurses_into_children() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let dir = tmp.path().join("src");
    let file_a = dir.join("a.rs");
    let file_b = dir.join("b.rs");
    write_file(&file_a, "fn a() {}");
    write_file(&file_b, "fn b() {}");

    add(&dir).unwrap();

    let entries = read_index().unwrap();
    let paths: Vec<_> = entries.iter().map(|(_, p)| p.clone()).collect();

    assert!(paths.contains(&"src/a.rs".to_string()));
    assert!(paths.contains(&"src/b.rs".to_string()));
}

#[test]
fn add_updates_oid_for_modified_file() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("hello.txt");
    write_file(&file_path, "v1");

    add(&file_path).unwrap();
    let entries_v1 = read_index().unwrap();
    let oid_v1 = entries_v1.iter()
        .find(|(_, p)| p == "hello.txt")
        .unwrap().0.clone();

    // Modify file and add again
    write_file(&file_path, "v2");
    add(&file_path).unwrap();
    let entries_v2 = read_index().unwrap();
    let oid_v2 = entries_v2.iter()
        .find(|(_, p)| p == "hello.txt")
        .unwrap().0.clone();

    assert_ne!(oid_v1, oid_v2, "OID should change when file content changes");
}

#[test]
fn add_is_idempotent_for_unmodified_file() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("hello.txt");
    write_file(&file_path, "static content");

    add(&file_path).unwrap();
    let entries_first = read_index().unwrap();

    add(&file_path).unwrap();
    let entries_second = read_index().unwrap();

    assert_eq!(entries_first, entries_second, "Index should not duplicate entries");
}

#[test]
fn add_multiple_files_writes_all_entries_once() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file1 = tmp.path().join("a.txt");
    let file2 = tmp.path().join("b.txt");
    write_file(&file1, "aaa");
    write_file(&file2, "bbb");

    add(&file1).unwrap();
    add(&file2).unwrap();

    let entries = read_index().unwrap();
    let paths: Vec<_> = entries.iter().map(|(_, p)| p.clone()).collect();

    assert!(paths.contains(&"a.txt".to_string()));
    assert!(paths.contains(&"b.txt".to_string()));
    assert_eq!(entries.len(), 2);
}
