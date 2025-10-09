use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{
    init::init,
    add::add,
    commit::commit,
    branch::branch,
};
use crate::core::repo::find_repo_root;
use crate::core::io::read_file;

fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

fn commit_helper(path: &Path, content: &str, message: &str) {
    fs::write(path, content).unwrap();
    add(path).unwrap();
    commit(message.to_string()).unwrap();
}

#[test]
fn branch_creates_new_branch_file_with_correct_oid() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("file.txt");
    commit_helper(&file_path, "data", "initial commit");

    branch("dev".to_string()).unwrap();

    let nag_dir = find_repo_root().unwrap().join(".nag");
    let main_branch_path = nag_dir.join("refs/heads/main");
    let dev_branch_path = nag_dir.join("refs/heads/dev");

    assert!(dev_branch_path.exists());

    let main_bytes = read_file(&main_branch_path.to_string_lossy());
    let main_oid = String::from_utf8_lossy(&main_bytes).trim().to_string();

    let dev_bytes = read_file(&dev_branch_path.to_string_lossy());
    let dev_oid = String::from_utf8_lossy(&dev_bytes).trim().to_string();

    assert_eq!(main_oid, dev_oid);
}

#[test]
fn branch_reports_when_branch_already_exists() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp); // silence unused warning

    let file_path = tmp.path().join("dup.txt");
    commit_helper(&file_path, "data", "first commit");

    branch("feature".to_string()).unwrap();
    branch("feature".to_string()).unwrap();

    let feature_path = find_repo_root().unwrap().join(".nag/refs/heads/feature");
    assert!(feature_path.exists());
}

#[test]
fn branch_does_not_affect_current_head() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    let file_path = tmp.path().join("check.txt");
    commit_helper(&file_path, "hello", "commit");

    branch("alt".to_string()).unwrap();

    let head_path = find_repo_root().unwrap().join(".nag/HEAD");
    let head_bytes = read_file(&head_path.to_string_lossy());
    let head_contents = String::from_utf8_lossy(&head_bytes);
    assert!(head_contents.contains("refs/heads/main"));
}

#[test]
fn branch_fails_gracefully_if_no_commits_yet() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    let result = branch("ghost".to_string());
    assert!(result.is_ok(), "should not panic — just create empty branch ref");

    let ghost_path = find_repo_root().unwrap().join(".nag/refs/heads/ghost");
    assert!(ghost_path.exists());

    let ghost_bytes = read_file(&ghost_path.to_string_lossy());
    let contents = String::from_utf8_lossy(&ghost_bytes);
    assert!(contents.trim().is_empty());
}

#[test]
fn branch_creates_correct_directory_structure_if_nested() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    let file_path = tmp.path().join("nested.txt");
    commit_helper(&file_path, "ok", "commit");

    branch("feature/ui".to_string()).unwrap();

    let branch_path = find_repo_root().unwrap().join(".nag/refs/heads/feature/ui");
    assert!(branch_path.exists());
}
