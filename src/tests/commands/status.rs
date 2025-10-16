use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{add::add, commit::commit, status::status};

fn init_repo(tmp: &TempDir) {
    std::env::set_current_dir(tmp.path()).unwrap();
    crate::commands::init::init(Some(tmp.path().to_string_lossy().to_string()));
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

// helper: stage + commit file with message
fn commit_helper(file_path: &Path, contents: &str, msg: &str) {
    write_file(file_path, contents);
    add(file_path).unwrap();
    commit(msg.to_string()).unwrap();
}

#[test]
fn status_reports_untracked() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("new.txt");
    write_file(&file_path, "hello");

    let out = status(false).unwrap();
    assert!(out.contains("Untracked files"));
    assert!(out.contains("new.txt"));
}

#[test]
fn status_reports_staged_before_commit() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("stage_me.txt");
    write_file(&file_path, "first version");
    add(&file_path).unwrap();

    let out = status(false).unwrap();
    assert!(out.contains("Staged:"));
    assert!(out.contains("Added Files:"));
    assert!(out.contains("stage_me.txt"));
}

#[test]
fn status_reports_modified() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("mod.txt");
    commit_helper(&file_path, "v1", "initial commit");

    // change after commit
    write_file(&file_path, "v2");

    let out = status(false).unwrap();
    assert!(out.contains("Unstaged:"));
    assert!(out.contains("Modified:"));
    assert!(out.contains("mod.txt"));
}

#[test]
fn status_reports_deleted() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("gone.txt");
    commit_helper(&file_path, "present", "add gone.txt");

    fs::remove_file(&file_path).unwrap();

    let out = status(false).unwrap();
    assert!(out.contains("Unstaged:"));
    assert!(out.contains("Deleted:"));
    assert!(out.contains("gone.txt"));
}

#[test]
fn status_reports_clean_repo() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let file_path = tmp.path().join("clean_repo.txt");
    commit_helper(&file_path, "content", "initial commit");

    let out = status(false).unwrap();
    assert!(!out.contains("Untracked files"));
    assert!(!out.contains("Unstaged:"));
    assert!(!out.contains("Added Files:"));
    assert!(!out.contains("Modified:"));
    assert!(!out.contains("Deleted:"));
}
