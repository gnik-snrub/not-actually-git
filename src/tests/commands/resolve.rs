use tempfile::TempDir;
use std::fs;
use std::fs::write;

use crate::commands::{
    add::add,
    branch::branch,
    checkout::checkout,
    commit::commit,
    merge::merge,
    resolve::resolve,
    status::status,
};
use crate::core::index::{read_index, EntryType};
use crate::commands::init::init;

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
fn resolve_clears_conflict_and_updates_index() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("a.txt");

    // main: a.txt = "one"
    commit_helper(&file, "one", "c1");

    // create feature branch
    branch("feature".to_string(), None).unwrap();
    checkout("feature".to_string()).unwrap();

    // feature: a.txt = "two"
    commit_helper(&file, "two", "c2");

    // back to main, diverge to create conflict
    checkout("main".to_string()).unwrap();
    fs::write(&file, "THREE").unwrap();
    add(&file).unwrap();
    commit("c3".to_string()).unwrap();

    // merging feature -> conflict expected
    assert!(merge("feature".to_string()).is_err());

    assert!(!status(false).unwrap().is_empty(), "Repo should be dirty before resolve");

    // resolve by picking a manual value
    fs::write(&file, "RESOLVED").unwrap();
    resolve("a.txt").unwrap();

    // index should now have a clean C entry with a single oid
    let index = read_index().unwrap();
    let entry = index.iter().find(|e| e.path == "a.txt").unwrap();

    assert_eq!(entry.entry_type, EntryType::C);
    assert_eq!(entry.oids.len(), 1);
}

#[test]
fn resolve_errors_on_missing_path() {
    let tmp = TempDir::new().unwrap();
    let _root = init_test_repo(&tmp);

    let err = resolve("nope.txt").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn resolve_only_modifies_target_entry() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file_a = root.join("a.txt");
    let file_b = root.join("b.txt");

    // commit both files
    commit_helper(&file_a, "AAA", "c1");
    commit_helper(&file_b, "BBB", "c2");

    // manually mark a.txt as conflict (simulate merge)
    {
        let mut index = read_index().unwrap();
        let a = index.iter_mut().find(|e| e.path == "a.txt").unwrap();
        a.entry_type = EntryType::X;
        fs::write(&root.join(".nag").join("index"), "").unwrap();
        crate::core::index::write_index(&index).unwrap();
    }

    // resolve only a.txt
    write(&file_a, "FIXED_A").unwrap();
    resolve("a.txt").unwrap();

    let index = read_index().unwrap();
    let a = index.iter().find(|e| e.path == "a.txt").unwrap();
    let b = index.iter().find(|e| e.path == "b.txt").unwrap();

    assert_eq!(a.entry_type, EntryType::C);
    assert_ne!(a.oids.len(), 0);

    // b.txt must remain unchanged
    assert_eq!(b.entry_type, EntryType::C);
}
