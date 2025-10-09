use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{
    init::init,
    add::add,
    commit::commit,
    branch::branch,
    checkout::checkout,
    status::status,
};
use crate::core::repo::find_repo_root;
use crate::core::io::read_file;

// Helper: create and cd into initialized repo
fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path)); // no unwrap, since init returns ()
    tmp.path().to_path_buf()
}

// Helper: write + add + commit a file
fn commit_helper(path: &Path, content: &str, message: &str) {
    fs::write(path, content).unwrap();
    add(path).unwrap();
    commit(message.to_string()).unwrap();
}

#[test]
fn checkout_switches_branches_cleanly() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let main_file = root.join("main.txt");
    commit_helper(&main_file, "main branch data", "initial commit");

    branch("feature".to_string()).unwrap();
    checkout("feature".to_string()).unwrap();

    let new_file = root.join("feature.txt");
    commit_helper(&new_file, "feature branch data", "feature commit");

    checkout("main".to_string()).unwrap();
    assert!(main_file.exists());
    assert!(!new_file.exists());
}

#[test]
fn checkout_refuses_to_overwrite_dirty_working_dir() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("restore_me.txt");
    commit_helper(&file_path, "old data", "initial commit");

    // make uncommitted edit
    fs::write(&file_path, "new data").unwrap();
    branch("restore".to_string()).unwrap();

    // this should fail, not succeed
    let result = checkout("restore".to_string());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("un-committed changes"));
}

#[test]
fn checkout_fails_on_nonexistent_branch() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("dummy.txt");
    commit_helper(&file_path, "some data", "commit");

    let result = checkout("notabranch".to_string());
    assert!(result.is_err());
}

#[test]
fn checkout_refuses_with_uncommitted_changes() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("warn.txt");
    commit_helper(&file_path, "clean", "commit");

    fs::write(&file_path, "dirty change").unwrap();

    let result = checkout("main".to_string());
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("un-committed changes"));
}

#[test]
fn checkout_updates_head_to_target_branch() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("main_file.txt");
    commit_helper(&file_path, "abc", "initial");

    branch("dev".to_string()).unwrap();
    checkout("dev".to_string()).unwrap();

    let head_path = find_repo_root().unwrap().join(".nag/HEAD");
    let bytes = read_file(&head_path.to_string_lossy());
    let head_contents = String::from_utf8_lossy(&bytes);
    assert!(head_contents.contains("refs/heads/dev"));
}

#[test]
fn checkout_produces_clean_status_after_switch() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let main_file = root.join("main.txt");
    commit_helper(&main_file, "stable", "init");

    branch("feature".to_string()).unwrap();
    checkout("feature".to_string()).unwrap();

    let out = status(false).unwrap();
    assert!(!out.contains("Untracked"));
    assert!(!out.contains("Modified"));
    assert!(!out.contains("Deleted"));
}
