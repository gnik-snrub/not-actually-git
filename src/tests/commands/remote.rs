use tempfile::TempDir;
use std::fs;
use crate::commands::remote::{ add_remote, remove_remote, fetch_remote };
use crate::core::repo::find_repo_root;
use crate::core::io::read_file;
use crate::commands::init::init;
use crate::commands::{
    add::add,
    commit::commit,
};

use std::env;
use std::path::Path;

fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

fn commit_helper(path: &Path, content: &str, msg: &str) {
    fs::write(path, content).unwrap();
    add(path).unwrap();
    commit(msg.to_string()).unwrap();
}

fn remote_commit_helper(repo_root: &Path, path: &Path, content: &str, msg: &str) {
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo_root).unwrap();

    commit_helper(path, content, msg);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn add_remote_creates_ref_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp); // real repo

    let repo_path = root.to_string_lossy().to_string();
    add_remote("origin".into(), repo_path.clone()).unwrap();

    let path = root.join(".nag/remotes/origin");
    assert!(path.exists());

    let contents = read_file(&path.to_string_lossy()).unwrap();
    assert_eq!(String::from_utf8_lossy(&contents).trim(), repo_path);
}

#[test]
fn add_remote_overwrites_existing() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let repo_path = root.to_string_lossy().to_string();

    add_remote("x".into(), repo_path.clone()).unwrap();
    add_remote("x".into(), repo_path.clone()).unwrap();

    let path = root.join(".nag/remotes/x");
    let contents = read_file(&path.to_string_lossy()).unwrap();
    assert_eq!(String::from_utf8_lossy(&contents).trim(), repo_path);
}

#[test]
fn remove_remote_deletes_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let repo_path = root.to_string_lossy().to_string();

    add_remote("r".into(), repo_path.clone()).unwrap();
    let path = root.join(".nag/remotes/r");
    assert!(path.exists());

    remove_remote("r".into()).unwrap();
    assert!(!path.exists());
}

#[test]
fn add_remote_errors_if_not_a_nag_repo() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(&tmp);

    let res = add_remote("bad".into(), "/definitely/not/a/repo".into());
    assert!(res.is_err());
}

#[test]
fn remove_remote_errors_if_missing() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(&tmp);

    let res = remove_remote("nope".into());
    assert!(res.is_err());
}

#[test]
fn add_and_remove_remote_do_not_affect_branches() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let branch_path = root.join(".nag/refs/heads/main");
    assert!(branch_path.exists());

    let repo_path = root.to_string_lossy().to_string();

    add_remote("o".into(), repo_path.clone()).unwrap();
    remove_remote("o".into()).unwrap();

    assert!(branch_path.exists());
}

#[test]
fn fetch_copies_new_branch_commit_and_tree() {
    // LOCAL repo
    let tmp_local = TempDir::new().unwrap();
    let local_root = init_test_repo(&tmp_local);

    // REMOTE repo
    let tmp_remote = TempDir::new().unwrap();
    let remote_root = init_test_repo(&tmp_remote);

    std::env::set_current_dir(local_root.to_path_buf()).unwrap();

    // Make remote content
    let file = remote_root.join("hello.txt");
    remote_commit_helper(&remote_root, &file, "hi", "initial");

    // Get remote's HEAD commit OID
    let remote_head_path = remote_root.join(".nag/refs/heads/main");
    let remote_oid = String::from_utf8_lossy(
        &read_file(&remote_head_path.to_string_lossy()).unwrap()
    ).trim().to_string();

    // Add remote to local
    add_remote("origin".into(), remote_root.to_string_lossy().to_string()).unwrap();

    // Fetch remote
    fetch_remote("origin".into()).unwrap();

    // Confirm local now has remote's commit object
    let local_commit_path = local_root.join(".nag/objects").join(&remote_oid);
    assert!(local_commit_path.exists(), "Local repo should contain remote commit");

    // Confirm remote-tracking ref exists
    let tracking_ref = local_root.join(".nag/refs/remotes/origin/main");
    assert!(tracking_ref.exists(), "Remote-tracking ref must be written");

    let written_oid = String::from_utf8_lossy(
        &read_file(&tracking_ref.to_string_lossy()).unwrap()
    ).trim().to_string();

    assert_eq!(written_oid, remote_oid, "Tracking ref must match remote HEAD");
}

#[test]
fn fetch_is_idempotent_and_does_not_duplicate_objects() {
    // Setup local + remote
    let tmp_local = TempDir::new().unwrap();
    let local_root = init_test_repo(&tmp_local);

    let tmp_remote = TempDir::new().unwrap();
    let remote_root = init_test_repo(&tmp_remote);

    // Commit something in remote
    let file = remote_root.join("t.txt");
    commit_helper(&file, "abc", "msg");

    // Add remote + first fetch
    add_remote("o".into(), remote_root.to_string_lossy().to_string()).unwrap();
    fetch_remote("o".into()).unwrap();

    // Count local objects after first fetch
    let local_obj_dir = local_root.join(".nag/objects");
    let first_count = fs::read_dir(&local_obj_dir).unwrap().count();

    // Fetch again — should not duplicate anything
    fetch_remote("o".into()).unwrap();
    let second_count = fs::read_dir(&local_obj_dir).unwrap().count();

    assert_eq!(first_count, second_count, "Fetch must be idempotent");
}

#[test]
fn fetch_copies_parent_commits() {
    // LOCAL repo
    let tmp_local = TempDir::new().unwrap();
    let local_root = init_test_repo(&tmp_local);

    // REMOTE repo
    let tmp_remote = TempDir::new().unwrap();
    let remote_root = init_test_repo(&tmp_remote);

    std::env::set_current_dir(local_root.to_path_buf()).unwrap();

    // Remote: create two commits (so HEAD has a parent)
    let file = remote_root.join("f.txt");
    remote_commit_helper(&remote_root, &file, "v1", "first");
    remote_commit_helper(&remote_root, &file, "v2", "second");

    // Read remote HEAD and its parent
    let remote_head_path = remote_root.join(".nag/refs/heads/main");
    let head_oid = String::from_utf8_lossy(&read_file(&remote_head_path.to_string_lossy()).unwrap()).trim().to_string();

    // Now read the COMMIT OBJECT itself
    let commit_obj_path = remote_root.join(".nag/objects").join(&head_oid);
    let commit_bytes = read_file(&commit_obj_path.to_string_lossy()).unwrap();
    let commit_text = String::from_utf8_lossy(&commit_bytes);

    // Extract parent line if present
    let parent_oid = commit_text
        .lines()
        .find(|l| l.starts_with("parent "))
        .expect("Commit should have a parent")
        .replacen("parent ", "", 1)
        .trim()
        .to_string();

    // Add remote + fetch
    add_remote("r".into(), remote_root.to_string_lossy().to_string()).unwrap();
    fetch_remote("r".into()).unwrap();

    // Confirm parent commit also exists locally
    let parent_path = local_root.join(".nag/objects").join(&parent_oid);
    assert!(parent_path.exists(), "Fetch must copy parent commit objects");
}

#[test]
fn fetch_multiple_branches() {
    // LOCAL
    let tmp_local = TempDir::new().unwrap();
    let local_root = init_test_repo(&tmp_local);

    // REMOTE
    let tmp_remote = TempDir::new().unwrap();
    let remote_root = init_test_repo(&tmp_remote);

    std::env::set_current_dir(local_root.to_path_buf()).unwrap();

    // Create second remote branch
    let file = remote_root.join("y.txt");
    remote_commit_helper(&remote_root, &file, "data", "first");

    // remote/main OID
    let main_oid = String::from_utf8_lossy(
        &read_file(&remote_root.join(".nag/refs/heads/main").to_string_lossy()).unwrap()
    ).trim().to_string();

    // Create branch "dev"
    fs::write(
        remote_root.join(".nag/refs/heads/dev"),
        main_oid.as_bytes()
    ).unwrap();

    // Add remote + fetch
    add_remote("origin".into(), remote_root.to_string_lossy().to_string()).unwrap();
    fetch_remote("origin".into()).unwrap();

    // Must have both tracking refs
    assert!(
        local_root.join(".nag/refs/remotes/origin/main").exists(),
        "main must be fetched"
    );

    assert!(
        local_root.join(".nag/refs/remotes/origin/dev").exists(),
        "dev must be fetched"
    );
}
