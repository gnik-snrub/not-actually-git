use tempfile::TempDir;
use std::fs;

use crate::commands::{
    init::init,
    add::add,
    commit::commit,
    tag::{ tag, list_tags, delete_tag },
};
use crate::core::io::read_file;
use crate::core::repo::find_repo_root;
use crate::core::hash::hash;

fn init_test_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

fn commit_helper(path: &std::path::Path, content: &str, message: &str) {
    fs::write(path, content).unwrap();
    add(path).unwrap();
    commit(message.to_string()).unwrap();
}

#[test]
fn tag_creates_lightweight_tag_for_head_commit() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("file.txt");
    commit_helper(&file, "v1", "first commit");

    tag(Some("v1".to_string()), None, None).unwrap();

    let tag_ref = find_repo_root().unwrap().join(".nag/refs/tags/v1");
    assert!(tag_ref.exists());

    let bytes = read_file(&tag_ref.to_string_lossy()).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert!(!contents.trim().is_empty());
}

#[test]
fn tag_creates_lightweight_tag_for_specific_commit() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("file.txt");
    commit_helper(&file, "v1", "first commit");
    let first_commit_oid = String::from_utf8_lossy(
        &read_file(&find_repo_root().unwrap().join(".nag/refs/heads/main").to_string_lossy()).unwrap()
    ).trim().to_string();

    commit_helper(&file, "v2", "second commit");

    tag(Some("old".to_string()), Some(first_commit_oid.clone()), None).unwrap();

    let tag_ref = find_repo_root().unwrap().join(".nag/refs/tags/old");
    let bytes = read_file(&tag_ref.to_string_lossy()).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert_eq!(contents.trim(), first_commit_oid);
}

#[test]
fn tag_creates_annotated_tag_with_message() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("note.txt");
    commit_helper(&file, "data", "commit message");

    tag(Some("annotated".to_string()), None, Some("stable release".to_string())).unwrap();

    let tag_ref = find_repo_root().unwrap().join(".nag/refs/tags/annotated");
    assert!(tag_ref.exists());

    let tag_oid = String::from_utf8_lossy(&read_file(&tag_ref.to_string_lossy()).unwrap()).trim().to_string();
    let tag_object_path = find_repo_root().unwrap().join(".nag/objects").join(&tag_oid);
    assert!(tag_object_path.exists());

    let bytes = read_file(&tag_object_path.to_string_lossy()).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert!(contents.contains("object "));
    assert!(contents.contains("stable release"));
}

#[test]
fn list_tags_returns_all_existing_tags() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("tag.txt");
    commit_helper(&file, "v1", "commit");

    tag(Some("alpha".to_string()), None, None).unwrap();
    tag(Some("beta".to_string()), None, Some("annotated".to_string())).unwrap();

    let list = list_tags(false).unwrap();
    assert!(list.contains("alpha"));
    assert!(list.contains("beta"));
}

#[test]
fn delete_tag_removes_tag_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("data.txt");
    commit_helper(&file, "x", "commit");

    tag(Some("temp".to_string()), None, None).unwrap();
    let tag_ref = find_repo_root().unwrap().join(".nag/refs/tags/temp");
    assert!(tag_ref.exists());

    delete_tag("temp".to_string()).unwrap();
    assert!(!tag_ref.exists());
}

#[test]
fn tag_overwrites_existing_tag() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("data.txt");
    commit_helper(&file, "first", "c1");
    tag(Some("release".to_string()), None, None).unwrap();

    commit_helper(&file, "second", "c2");
    let new_oid = String::from_utf8_lossy(
        &read_file(&find_repo_root().unwrap().join(".nag/refs/heads/main").to_string_lossy()).unwrap()
    ).trim().to_string();

    tag(Some("release".to_string()), Some(new_oid.clone()), None).unwrap();

    let tag_ref = find_repo_root().unwrap().join(".nag/refs/tags/release");
    let bytes = read_file(&tag_ref.to_string_lossy()).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert_eq!(contents.trim(), new_oid);
}

#[test]
fn tag_fails_when_commit_oid_does_not_exist() {
    // Purpose: verify tagging fails cleanly when the specified commit object doesn't exist
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    // Using a fake OID that will not exist in .nag/objects
    let fake_oid = "deadbeef1234567890abcdef";

    let result = tag(Some("badtag".to_string()), Some(fake_oid.to_string()), None);

    // Expect an Err, not a panic
    assert!(result.is_err(), "Expected error when tagging nonexistent commit");

    // And ensure that no tag file was created
    let tag_ref = find_repo_root().unwrap().join(".nag/refs/tags/badtag");
    assert!(!tag_ref.exists(), "No tag file should be created for missing commit");
}

