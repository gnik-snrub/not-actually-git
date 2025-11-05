use tempfile::TempDir;
use std::fs;
use crate::commands::{init::init, add::add, commit::commit, branch::branch, checkout::checkout};
use crate::core::repo::find_repo_root;
use crate::core::io::{read_file, write_file};
use crate::commands::merge::merge;

// helper
fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

fn commit_helper(path: &std::path::Path, content: &str, msg: &str) {
    fs::write(path, content).unwrap();
    add(path).unwrap();
    commit(msg.to_string()).unwrap();
}

#[test]
fn ff_merge_succeeds_on_direct_descendant() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("file.txt");
    commit_helper(&file, "v1", "first commit");

    branch("feature".to_string(), None).unwrap();
    checkout("feature".to_string()).unwrap();

    commit_helper(&file, "v2", "second commit");

    add(&file).unwrap();
    commit("sync before merge".to_string()).unwrap();
    checkout("main".to_string()).unwrap();
    merge("feature".to_string()).unwrap();

    // both branches now share same oid
    let nag = find_repo_root().unwrap().join(".nag");
    let main_ref = nag.join("refs/heads/main");
    let feature_ref = nag.join("refs/heads/feature");

    let main_oid = String::from_utf8_lossy(&read_file(&main_ref.to_string_lossy()).unwrap()).trim().to_string();
    let feat_oid = String::from_utf8_lossy(&read_file(&feature_ref.to_string_lossy()).unwrap()).trim().to_string();
    assert_eq!(main_oid, feat_oid);
}

#[test]
fn ff_merge_reports_already_up_to_date() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("a.txt");
    commit_helper(&file, "init", "base commit");

    let result = merge("main".to_string());
    assert!(result.is_ok());
}

#[test]
fn ff_merge_fails_on_diverged_history() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("file.txt");
    commit_helper(&file, "base", "base commit");

    branch("alt".to_string(), None).unwrap();
    checkout("alt".to_string()).unwrap();
    commit_helper(&file, "alt change", "alt commit");

    checkout("main".to_string()).unwrap();
    commit_helper(&file, "main change", "main commit");

    let result = merge("alt".to_string());
    assert!(result.is_err());
}

#[test]
fn ff_merge_fails_on_dirty_working_directory() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("dirty.txt");
    commit_helper(&file, "clean", "commit clean");

    branch("dirty".to_string(), None).unwrap();
    checkout("dirty".to_string()).unwrap();
    commit_helper(&file, "new", "new commit");

    checkout("main".to_string()).unwrap();
    fs::write(&file, "unsaved").unwrap(); // dirty

    let result = merge("dirty".to_string());
    assert!(result.is_err());
    assert!(format!("{:?}", result.unwrap_err()).contains("not clean"));
}

#[test]
fn ff_merge_fails_on_detached_head() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("detached.txt");
    commit_helper(&file, "data", "initial");

    // manually detach HEAD
    let nag = find_repo_root().unwrap().join(".nag");
    let head_path = nag.join("HEAD");
    let oid = String::from_utf8_lossy(&read_file(&nag.join("refs/heads/main").to_string_lossy()).unwrap()).trim().to_string();
    write_file(&oid.as_bytes().to_vec(), &head_path).unwrap();

    let result = merge("main".to_string());
    assert!(result.is_err());
}
