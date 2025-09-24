use std::fs;
use tempfile::TempDir;

use crate::core::index::{read_index, write_index};

// Helper: initialize fake repo with .nag structure and cd into it
fn init_repo(tmp: &TempDir) -> std::path::PathBuf {
    let nag_root = tmp.path().join(".nag");
    let objects = nag_root.join("objects");
    fs::create_dir_all(&objects).unwrap();

    // important: pretend we’re “inside” the repo root
    std::env::set_current_dir(tmp.path()).unwrap();

    nag_root
}

#[test]
fn read_index_success() {
    let tmp = TempDir::new().unwrap();
    let nag_dir = init_repo(&tmp);

    let index_path = nag_dir.join("index");
    fs::write(&index_path, "abc123\tfile.txt\nxyz789\tsrc/main.rs\n").unwrap();

    let entries = read_index().unwrap();
    assert_eq!(
        entries,
        vec![
            ("abc123".to_string(), "file.txt".to_string()),
            ("xyz789".to_string(), "src/main.rs".to_string())
        ]
    );
}

#[test]
fn read_index_not_found() {
    let tmp = TempDir::new().unwrap();
    let nag_dir = init_repo(&tmp);

    let index_path = nag_dir.join("index");
    assert!(!index_path.exists());

    let index = read_index().unwrap();
    assert!(index.is_empty(), "expected empty index when none exists");
}

#[test]
fn read_index_ignores_malformed_lines() {
    let tmp = TempDir::new().unwrap();
    let nag_dir = init_repo(&tmp);

    let index_path = nag_dir.join("index");
    fs::write(&index_path, "goodoid\tgoodpath\nmalformedline\n").unwrap();

    let entries = read_index().unwrap();
    assert_eq!(entries, vec![("goodoid".to_string(), "goodpath".to_string())]);
}

#[test]
fn write_index_success() {
    let tmp = TempDir::new().unwrap();
    let nag_dir = init_repo(&tmp);

    let entries = vec![
        ("abc123".to_string(), "file.txt".to_string()),
        ("xyz789".to_string(), "src/main.rs".to_string()),
    ];

    write_index(&entries).unwrap();
    let index_path = nag_dir.join("index");
    let contents = fs::read_to_string(index_path).unwrap();

    assert_eq!(contents, "abc123\tfile.txt\nxyz789\tsrc/main.rs\n");
}

#[test]
fn write_index_empty_creates_file() {
    let tmp = TempDir::new().unwrap();
    let nag_dir = init_repo(&tmp);

    let entries: Vec<(String, String)> = vec![];
    write_index(&entries).unwrap();

    let index_path = nag_dir.join("index");
    let contents = fs::read_to_string(index_path).unwrap();

    assert_eq!(contents, "");
}

#[test]
fn write_and_read_round_trip() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let entries = vec![
        ("abc123".to_string(), "file.txt".to_string()),
        ("xyz789".to_string(), "src/main.rs".to_string()),
    ];

    write_index(&entries).unwrap();
    let read_back = read_index().unwrap();

    assert_eq!(entries, read_back);
}
