use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{
    init::init,
    add::add,
    commit::commit,
    branch::{branch, branch_list},
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

    branch("dev".to_string(), None).unwrap();

    let nag_dir = find_repo_root().unwrap().join(".nag");
    let main_branch_path = nag_dir.join("refs/heads/main");
    let dev_branch_path = nag_dir.join("refs/heads/dev");

    assert!(dev_branch_path.exists());

    let main_bytes = read_file(&main_branch_path.to_string_lossy()).unwrap();
    let main_oid = String::from_utf8_lossy(&main_bytes).trim().to_string();

    let dev_bytes = read_file(&dev_branch_path.to_string_lossy()).unwrap();
    let dev_oid = String::from_utf8_lossy(&dev_bytes).trim().to_string();

    assert_eq!(main_oid, dev_oid);
}

#[test]
fn branch_can_be_created_from_specific_oid() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    // Make two commits
    let file = root.join("specific.txt");
    commit_helper(&file, "v1", "first commit");
    fs::write(&file, "v2").unwrap();
    add(&file).unwrap();
    commit("second commit".to_string()).unwrap();

    // Get OID of first commit
    let main_path = find_repo_root().unwrap().join(".nag/refs/heads/main");
    let main_bytes = read_file(&main_path.to_string_lossy()).unwrap();
    let first_commit_oid = String::from_utf8_lossy(&main_bytes).trim().to_string();

    // Create branch from specific commit OID
    branch("retro".to_string(), Some(first_commit_oid.clone())).unwrap();

    let retro_path = find_repo_root().unwrap().join(".nag/refs/heads/retro");
    assert!(retro_path.exists());

    let retro_bytes = read_file(&retro_path.to_string_lossy()).unwrap();
    let retro_oid = String::from_utf8_lossy(&retro_bytes).trim().to_string();

    assert_eq!(retro_oid, first_commit_oid);
}

#[test]
fn branch_fails_on_duplicate_name() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("dup.txt");
    commit_helper(&file_path, "data", "first commit");

    branch("feature".to_string(), None).unwrap();
    let result = branch("feature".to_string(), None);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().kind(),
        std::io::ErrorKind::AlreadyExists
    );
}

#[test]
fn branch_list_shows_all_and_marks_current() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_path = root.join("alpha.txt");
    commit_helper(&file_path, "content", "init");

    branch("beta".to_string(), None).unwrap();
    branch("gamma".to_string(), None).unwrap();

    let output = branch_list(false).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    let _ = branch_list(true);

    assert!(lines.contains(&"*main"));
    assert!(lines.contains(&"beta"));
    assert!(lines.contains(&"gamma"));
    assert_eq!(lines, vec!["beta", "gamma", "*main"]);
}

#[test]
fn branch_does_not_affect_current_head() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    let file_path = tmp.path().join("check.txt");
    commit_helper(&file_path, "hello", "commit");

    branch("alt".to_string(), None).unwrap();

    let head_path = find_repo_root().unwrap().join(".nag/HEAD");
    let head_bytes = read_file(&head_path.to_string_lossy()).unwrap();
    let head_contents = String::from_utf8_lossy(&head_bytes);
    assert!(head_contents.contains("refs/heads/main"));
}

#[test]
fn branch_creates_correct_directory_structure_if_nested() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    let file_path = tmp.path().join("nested.txt");
    commit_helper(&file_path, "ok", "commit");

    branch("feature/ui".to_string(), None).unwrap();

    let branch_path = find_repo_root().unwrap().join(".nag/refs/heads/feature/ui");
    assert!(branch_path.exists());

    let list_out = branch_list(false).unwrap();
    assert!(list_out.contains("ui"));
    assert!(list_out.contains("feature"));
}
