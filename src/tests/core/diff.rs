use tempfile::TempDir;
use std::fs;
use std::path::Path;

use crate::commands::{init::init, add::add, commit::commit};
use crate::core::diff::{diff_working_to_index, diff_index_to_head, get_all_diffs, DiffType};

fn init_repo(tmp: &TempDir) -> std::path::PathBuf {
    std::env::set_current_dir(tmp.path()).unwrap();
    let repo_path = tmp.path().to_string_lossy().to_string();
    init(Some(repo_path));
    tmp.path().to_path_buf()
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn commit_helper(path: &Path, content: &str, msg: &str) {
    write_file(path, content);
    add(path).unwrap();
    commit(msg.to_string()).unwrap();
}

#[test]
fn diff_working_to_index_detects_untracked() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    let file = root.join("untracked.txt");
    write_file(&file, "hello");

    let diffs = diff_working_to_index().unwrap();
    let untracked = diffs.get(&DiffType::Untracked).unwrap();
    assert!(untracked.contains(&"untracked.txt".to_string()));
}

#[test]
fn diff_working_to_index_detects_modified_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    let file = root.join("mod.txt");
    commit_helper(&file, "v1", "initial commit");
    write_file(&file, "v2");

    let diffs = diff_working_to_index().unwrap();
    let modified = diffs.get(&DiffType::Modified).unwrap();
    assert!(modified.contains(&"mod.txt".to_string()));
}

#[test]
fn diff_index_to_head_detects_added_file_in_index() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    let file = root.join("added.txt");
    write_file(&file, "content");
    add(&file).unwrap();

    let diffs = diff_index_to_head().unwrap();
    let added = diffs.get(&DiffType::Added).unwrap();
    assert!(added.contains(&"added.txt".to_string()));
}

#[test]
fn diff_index_to_head_detects_staged_modified_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    let file = root.join("staged_mod.txt");
    commit_helper(&file, "old", "base commit");

    write_file(&file, "new");
    add(&file).unwrap();

    let diffs = diff_index_to_head().unwrap();
    let staged = diffs.get(&DiffType::Staged).unwrap();
    assert!(staged.contains(&"staged_mod.txt".to_string()));
}

#[test]
fn diff_index_to_head_detects_deleted_index_file() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    let file = root.join("deleted.txt");
    commit_helper(&file, "exists", "initial");

    fs::remove_file(&file).unwrap();

    let diffs = diff_index_to_head().unwrap();
    let deleted = diffs.get(&DiffType::Deleted).unwrap();
    assert!(deleted.contains(&"deleted.txt".to_string()));
}

#[test]
fn diff_index_to_head_detects_staged_delete_against_head() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    let file = root.join("old.txt");
    commit_helper(&file, "exists", "initial");

    // simulate staged deletion: remove + add
    fs::remove_file(&file).unwrap();
    add(&file).unwrap(); // this stages the deletion

    let diffs = diff_index_to_head().unwrap();
    let staged_delete = diffs.get(&DiffType::StagedDelete).unwrap();
    assert!(staged_delete.contains(&"old.txt".to_string()));
}

#[test]
fn get_all_diffs_combines_results() {
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    // Untracked file
    let untracked = root.join("new.txt");
    write_file(&untracked, "test");

    // Modified file
    let modified = root.join("mod.txt");
    commit_helper(&modified, "v1", "init");
    write_file(&modified, "v2");

    // Staged file
    let staged = root.join("add.txt");
    write_file(&staged, "stage me");
    add(&staged).unwrap();

    let diffs = get_all_diffs().unwrap();

    assert!(diffs.get(&DiffType::Untracked).unwrap().contains(&"new.txt".to_string()));
    assert!(diffs.get(&DiffType::Modified).unwrap().contains(&"mod.txt".to_string()));
    assert!(diffs.get(&DiffType::Added).unwrap().contains(&"add.txt".to_string()));
}

#[test]
fn diff_excludes_ignored_files() {
    // Purpose: ensure ignored files never appear in diff results
    let tmp = TempDir::new().unwrap();
    let root = init_repo(&tmp);

    std::fs::write(root.join(".nagignore"), "*.tmp\n").unwrap();

    let tracked = root.join("main.rs");
    let ignored = root.join("build.tmp");

    // commit tracked, leave ignored untracked
    commit_helper(&tracked, "fn main() {}", "initial commit");

    // modify both after commit
    std::fs::write(&tracked, "fn main() { println!(\"hi\"); }").unwrap();
    std::fs::write(&ignored, "debug build output").unwrap();

    let diffs = get_all_diffs().unwrap();

    for (_, files) in diffs {
        for f in files {
            assert!(!f.contains("build.tmp"));
        }
    }
}
