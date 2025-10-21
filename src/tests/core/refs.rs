use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::core::refs::{
    resolve_head,
    read_ref,
    update_ref,
};
use crate::core::repo::find_repo_root;
use crate::core::io::read_file;

fn init_fake_repo(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    let nag_dir = root.join(".nag/refs/heads");
    fs::create_dir_all(&nag_dir).unwrap();
    std::env::set_current_dir(&root).unwrap();
    root
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn resolve_head_returns_branch_and_oid_when_symbolic() {
    let tmp = TempDir::new().unwrap();
    let root = init_fake_repo(&tmp);

    let head_path = root.join(".nag/HEAD");
    let branch_path = root.join(".nag/refs/heads/main");

    write(&head_path, "ref: refs/heads/main");
    write(&branch_path, "abcd1234deadbeef");

    let (branch_name, oid) = resolve_head().unwrap();
    assert_eq!(branch_name, Some("main".to_string()));
    assert_eq!(oid, "abcd1234deadbeef");
}

#[test]
fn resolve_head_returns_none_and_oid_when_detached() {
    let tmp = TempDir::new().unwrap();
    let root = init_fake_repo(&tmp);

    let head_path = root.join(".nag/HEAD");
    write(&head_path, "cafebabe12345678");

    let (branch_name, oid) = resolve_head().unwrap();
    assert_eq!(branch_name, None);
    assert_eq!(oid, "cafebabe12345678");
}

#[test]
fn resolve_head_trims_newlines_and_whitespace() {
    let tmp = TempDir::new().unwrap();
    let root = init_fake_repo(&tmp);

    let head_path = root.join(".nag/HEAD");
    let branch_path = root.join(".nag/refs/heads/main");

    write(&head_path, "ref: refs/heads/main\n");
    write(&branch_path, "deadbeef1234abcd\n\n");

    let (branch_name, oid) = resolve_head().unwrap();
    assert_eq!(branch_name, Some("main".to_string()));
    assert_eq!(oid, "deadbeef1234abcd");
}

#[test]
fn read_ref_reads_full_ref_path() {
    let tmp = TempDir::new().unwrap();
    let root = init_fake_repo(&tmp);

    let full_ref_path = root.join(".nag/refs/heads/dev");
    write(&full_ref_path, "cafebabe1234abcd");

    let result = read_ref("refs/heads/dev").unwrap();
    assert_eq!(result, "cafebabe1234abcd");
}

#[test]
fn read_ref_reads_short_branch_name() {
    let tmp = TempDir::new().unwrap();
    let root = init_fake_repo(&tmp);

    let short_ref_path = root.join(".nag/refs/heads/main");
    write(&short_ref_path, "deadbeef9876feed");

    let result = read_ref("main").unwrap();
    assert_eq!(result, "deadbeef9876feed");
}

#[test]
fn read_ref_trims_whitespace_and_newlines() {
    let tmp = TempDir::new().unwrap();
    let root = init_fake_repo(&tmp);

    let path = root.join(".nag/refs/heads/feature");
    write(&path, "abcd1234efef5678\n\n");

    let result = read_ref("feature").unwrap();
    assert_eq!(result, "abcd1234efef5678");
}

#[test]
fn read_ref_fails_on_missing_ref() {
    let tmp = TempDir::new().unwrap();
    init_fake_repo(&tmp);

    let result = read_ref("nonexistent");
    assert!(result.is_err());
}

#[test]
fn update_ref_creates_and_writes_branch() {
    // Purpose: ensure update_ref creates parent dirs and writes correct OID
    let tmp = TempDir::new().unwrap();
    init_fake_repo(&tmp);

    let oid = "abc123";
    let ref_path = tmp.path().join(".nag/refs/heads/feature/test");

    // should create nested dirs and write oid
    update_ref("feature/test", oid).unwrap();
    assert!(ref_path.exists());

    let bytes = read_file(&ref_path.to_string_lossy()).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert_eq!(contents.trim(), oid);
}

#[test]
fn update_ref_overwrites_existing_branch() {
    // Purpose: ensure update_ref can overwrite an existing ref cleanly
    let tmp = TempDir::new().unwrap();
    init_fake_repo(&tmp);

    update_ref("main", "111111").unwrap();
    update_ref("main", "222222").unwrap();

    let ref_path = tmp.path().join(".nag/refs/heads/main");
    let bytes = read_file(&ref_path.to_string_lossy()).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert_eq!(contents.trim(), "222222");
}

