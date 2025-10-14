use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{
    init::init,
    add::add,
    commit::commit,
    restore::restore,
};
use crate::core::repo::find_repo_root;

// Helper: create and cd into initialized repo
fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

// Helper: write + add + commit a file
fn commit_helper(path: &Path, content: &str, message: &str) {
    fs::write(path, content).unwrap();
    add(path).unwrap();
    commit(message.to_string()).unwrap();
}

#[test]
fn restore_brings_back_deleted_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("hello.txt");
    commit_helper(&file_path, "hello world", "first commit");

    // delete the file manually
    fs::remove_file(&file_path).unwrap();
    assert!(!file_path.exists());

    // restore from HEAD
    restore("hello.txt".to_string()).unwrap();

    let restored = fs::read_to_string(&file_path).unwrap();
    assert_eq!(restored, "hello world");
}

#[test]
fn restore_recovers_entire_directory() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let dir = root.join("src");
    fs::create_dir_all(&dir).unwrap();

    let file_a = dir.join("main.rs");
    let file_b = dir.join("lib.rs");

    commit_helper(&file_a, "fn main() {}", "commit a");
    commit_helper(&file_b, "pub fn util() {}", "commit b");

    fs::remove_file(&file_a).unwrap();
    fs::remove_file(&file_b).unwrap();
    assert!(!file_a.exists() && !file_b.exists());

    restore("src".to_string()).unwrap();

    assert!(file_a.exists() && file_b.exists());
    assert_eq!(fs::read_to_string(&file_a).unwrap(), "fn main() {}");
    assert_eq!(fs::read_to_string(&file_b).unwrap(), "pub fn util() {}");
}

#[test]
fn restore_is_noop_when_file_matches_head() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("noop.txt");
    commit_helper(&file_path, "data", "commit");

    let before = fs::read_to_string(&file_path).unwrap();
    restore("noop.txt".to_string()).unwrap();
    let after = fs::read_to_string(&file_path).unwrap();

    assert_eq!(before, after);
}

#[test]
fn restore_fails_on_untracked_path() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("tracked.txt");
    commit_helper(&file_path, "ok", "commit");

    let result = restore("not_tracked.txt".to_string());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn restore_partially_missing_directory() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let dir = root.join("docs");
    fs::create_dir_all(&dir).unwrap();

    let a = dir.join("intro.txt");
    let b = dir.join("guide.txt");

    commit_helper(&a, "intro", "a");
    commit_helper(&b, "guide", "b");

    fs::remove_file(&a).unwrap(); // only delete one file
    assert!(!a.exists() && b.exists());

    restore("docs".to_string()).unwrap();

    assert!(a.exists() && b.exists());
    assert_eq!(fs::read_to_string(&a).unwrap(), "intro");
}
