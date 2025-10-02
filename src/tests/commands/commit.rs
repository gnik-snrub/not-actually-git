use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{init::init, add::add, commit::commit};
use crate::core::index::read_index;

// Helper: create a real repo via `init` and cd into it
fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();

    // convert PathBuf -> String
    let repo_path = tmp.path().to_string_lossy().to_string();
    crate::commands::init::init(Some(repo_path));

    tmp.path().to_path_buf()
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn first_commit_creates_commit_object_and_branch() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(&tmp);

    let file_path = tmp.path().join("a.txt");
    write_file(&file_path, "hello");
    add(&file_path).unwrap();
    commit("first".into()).unwrap();

    let branch_ref = tmp.path().join(".nag/refs/heads/main");
    let commit_oid = fs::read_to_string(&branch_ref).unwrap();
    assert!(!commit_oid.trim().is_empty(), "branch should point to a commit oid");

    let commit_obj = fs::read_to_string(tmp.path().join(".nag/objects").join(commit_oid.trim())).unwrap();
    assert!(commit_obj.contains("tree"), "commit object must contain a tree");
    assert!(commit_obj.contains("first"), "commit object must contain the commit message");
}

#[test]
fn commit_records_parent_oid() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(&tmp);

    let file_path = tmp.path().join("a.txt");
    write_file(&file_path, "hello");
    add(&file_path).unwrap();
    commit("first".into()).unwrap();

    let first_commit_oid = fs::read_to_string(tmp.path().join(".nag/refs/heads/main")).unwrap();

    write_file(&file_path, "goodbye");
    add(&file_path).unwrap();
    commit("second".into()).unwrap();

    let second_commit_oid = fs::read_to_string(tmp.path().join(".nag/refs/heads/main")).unwrap();
    let second_body = fs::read_to_string(tmp.path().join(".nag/objects").join(second_commit_oid.trim())).unwrap();

    assert!(second_body.contains(&format!("parent {}", first_commit_oid.trim())));
}

#[test]
fn commit_message_is_preserved() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(&tmp);

    let file_path = tmp.path().join("msg.txt");
    write_file(&file_path, "stuff");
    add(&file_path).unwrap();
    commit("special commit message".into()).unwrap();

    let commit_oid = fs::read_to_string(tmp.path().join(".nag/refs/heads/main")).unwrap();
    let body = fs::read_to_string(tmp.path().join(".nag/objects").join(commit_oid.trim())).unwrap();

    assert!(body.contains("special commit message"), "commit object should contain the commit message verbatim");
}
