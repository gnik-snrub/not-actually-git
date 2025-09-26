use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{add::add, status::status};

fn init_repo(tmp: &TempDir) {
    let nag_root = tmp.path().join(".nag/objects");
    fs::create_dir_all(&nag_root).unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn status_reports_untracked() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("new.txt");
    write_file(&file_path, "hello");

    let out = status().unwrap();
    assert!(out.contains("Untracked files"));
    assert!(out.contains("new.txt"));
}

#[test]
fn status_reports_staged_clean() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("clean.txt");
    write_file(&file_path, "stable");
    add(&file_path).unwrap();

    let out = status().unwrap();
    assert!(out.contains("Staged changes"));
    assert!(out.contains("clean.txt"));
}

#[test]
fn status_reports_modified() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("mod.txt");
    write_file(&file_path, "v1");
    add(&file_path).unwrap();

    // modify the file after staging
    write_file(&file_path, "v2");

    let out = status().unwrap();
    assert!(out.contains("Modified"));
    assert!(out.contains("mod.txt"));
}

#[test]
fn status_reports_deleted() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("gone.txt");
    write_file(&file_path, "gone");
    add(&file_path).unwrap();

    fs::remove_file(&file_path).unwrap();

    let out = status().unwrap();
    assert!(out.contains("Deleted"));
    assert!(out.contains("gone.txt"));
}

#[test]
fn status_reports_clean_repo() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("clean_repo.txt");
    write_file(&file_path, "nothing to see");
    add(&file_path).unwrap();

    let out = status().unwrap();
    // A clean repo should not show untracked/modified/deleted sections
    assert!(out.contains("Untracked files"));
    assert!(!out.contains("Modified"));
    assert!(!out.contains("Deleted"));
    assert!(out.contains("Staged changes")); // still lists staged files
}
