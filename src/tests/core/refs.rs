use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::core::refs::resolve_head;
use crate::core::repo::find_repo_root;

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
