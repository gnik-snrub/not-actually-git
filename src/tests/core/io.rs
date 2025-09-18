use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::core::io::{read_file, write_file};

#[test]
fn read_file_returns_contents() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("test.txt");

    fs::write(&file_path, b"hello world").unwrap();
    let bytes = read_file(file_path.to_str().unwrap());

    assert_eq!(bytes, b"hello world");
}

#[test]
fn read_file_returns_empty_on_missing() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("missing.txt");

    let bytes = read_file(file_path.to_str().unwrap());

    assert_eq!(bytes, Vec::<u8>::new());
}

#[test]
fn write_file_creates_blob() {
    let tmp = TempDir::new().unwrap();
    let hash = "abcd1234".to_string();

    let data = b"some data".to_vec();
    write_file(data.clone(), tmp.path(), &hash);

    let final_path = tmp.path().join("abcd1234.blob");
    assert!(final_path.exists());

    let stored = fs::read(final_path).unwrap();
    assert_eq!(stored, data);
}

#[test]
fn write_file_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let hash = "idempotent".to_string();
    let final_path = tmp.path().join("idempotent.blob");

    // first write
    write_file(b"first".to_vec(), tmp.path(), &hash);
    let first = fs::read(&final_path).unwrap();

    // second write with different data should be ignored
    write_file(b"second".to_vec(), tmp.path(), &hash);
    let second = fs::read(&final_path).unwrap();

    assert_eq!(first, second);
}

#[test]
fn write_file_leaves_no_temp_files() {
    let tmp = TempDir::new().unwrap();
    let hash = "checktemp".to_string();

    // Write a blob normally
    write_file(b"some data".to_vec(), tmp.path(), &hash);

    // Scan for leftover .tmp.* files
    let tmpfiles: Vec<PathBuf> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map(|s| s.to_string_lossy().contains("tmp")).unwrap_or(false))
        .collect();

    assert!(tmpfiles.is_empty(), "temp files were left behind: {:?}", tmpfiles);
}

