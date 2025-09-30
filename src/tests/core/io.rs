use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::core::io::{read_file, write_file, write_object};
use crate::core::hash::hash;
use crate::tests::common::setup_nag_repo;

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
fn write_object_creates_blob() {
    let tmp = TempDir::new().unwrap();
    let objects = setup_nag_repo(&tmp);

    let data = b"some data".to_vec();
    let oid = hash(&data);
    write_object(&data, &oid).unwrap();

    let final_path = objects.join(&oid);
    assert!(final_path.exists());

    let stored = fs::read(&final_path).unwrap();
    assert_eq!(stored, data);
}

#[test]
fn write_object_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let objects = setup_nag_repo(&tmp);

    let data = b"idempotent".to_vec();
    let oid = hash(&data);
    let final_path = objects.join(&oid);

    // first write
    write_object(&data, &oid).unwrap();
    let first = fs::read(&final_path).unwrap();

    // second write with bogus content shouldn't overwrite
    let bogus = b"different".to_vec();
    write_object(&bogus, &oid).unwrap();
    let second = fs::read(&final_path).unwrap();

    assert_eq!(first, second, "existing blob was overwritten when it shouldn't be");
}

#[test]
fn write_file_leaves_no_temp_files() {
    let tmp = TempDir::new().unwrap();
    setup_nag_repo(&tmp);

    let file_path = tmp.path().join("temp_test.txt");

    // Write a normal file with safe write
    write_file(&b"some data".to_vec(), &file_path).unwrap();

    // Verify file exists
    assert!(file_path.exists(), "final file was not created");

    // Scan for leftover temp files
    let tmpfiles: Vec<PathBuf> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension()
                .map(|s| s.to_string_lossy().contains("tmp"))
                .unwrap_or(false)
        })
        .collect();

    assert!(
        tmpfiles.is_empty(),
        "temp files were left behind: {:?}",
        tmpfiles
    );
}
