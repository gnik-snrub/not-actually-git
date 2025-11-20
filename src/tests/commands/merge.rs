use tempfile::TempDir;
use std::fs;
use crate::commands::{init::init, add::add, commit::commit, branch::branch, checkout::checkout, status::status};
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

#[test]
fn merge_simple_conflict() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("conflict.txt");
    commit_helper(&file, "base", "init");

    // Create feature branch
    branch("feature".to_string(), None).unwrap();

    // Modify on main
    commit_helper(&file, "main-change", "main change");

    // Switch to feature and modify differently
    checkout("feature".to_string()).unwrap();
    commit_helper(&file, "feature-change", "feature change");

    // Go back to main and try to merge feature - should create conflict
    checkout("main".to_string()).unwrap();
    let res = merge("feature".to_string());
    
    // Should succeed but create conflict markers
    assert!(res.is_err());
    // Could also check that the file has conflict markers or index has conflict entries
}

#[test]
fn merge_only_one_side_edits() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("solo.txt");
    commit_helper(&file, "base", "c1");

    // Create branch (it stays at current commit)
    branch("alt".to_string(), None).unwrap();

    // Only main edits (alt stays at base)
    commit_helper(&file, "main-edit", "main edit");

    // Merge alt (which has no changes) - should be already up-to-date or fast-forward back
    let res = merge("alt".to_string());
    assert!(res.is_ok());
}

#[test]
fn merge_both_add_same_file_conflict() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let base = root.join("base.txt");
    commit_helper(&base, "x", "c1");

    // Create branch
    branch("f".to_string(), None).unwrap();

    // Main adds new file
    let new_main = root.join("new.txt");
    commit_helper(&new_main, "main-new", "main add");

    // Switch to feature and add same file with different content
    checkout("f".to_string()).unwrap();
    let new_feature = root.join("new.txt");
    commit_helper(&new_feature, "feature-new", "feature add");

    // Go back to main and merge - should create conflict
    checkout("main".to_string()).unwrap();
    let res = merge("f".to_string());
    assert!(res.is_err()); // Merge runs but creates conflict
}

#[test]
fn merge_directory_merge() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let f1 = root.join("src/a.rs");
    fs::create_dir_all(f1.parent().unwrap()).unwrap();
    commit_helper(&f1, "v1", "init");

    // Create branch
    branch("feature".to_string(), None).unwrap();

    // Main adds new file in same dir
    let f2 = root.join("src/b.rs");
    commit_helper(&f2, "b content", "b add");

    // Merge feature (which has old state) - should be up-to-date
    let res = merge("feature".to_string());
    assert!(res.is_ok());
}

#[test]
fn merge_nested_directories() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let a = root.join("src/utils/a.txt");
    fs::create_dir_all(a.parent().unwrap()).unwrap();
    commit_helper(&a, "v1", "c1");

    // Create branch
    branch("other".to_string(), None).unwrap();

    // Main adds different nested file
    let b = root.join("src/utils/b.txt");
    commit_helper(&b, "v2", "c2");

    // Merge other (old state) - should be up-to-date
    let res = merge("other".to_string());
    assert!(res.is_ok());
}

#[test]
fn merge_delete_vs_edit_conflict() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let file = root.join("del.txt");
    commit_helper(&file, "base", "init");

    // Create branch
    branch("alt".to_string(), None).unwrap();

    // Main deletes
    fs::remove_file(&file).unwrap();
    add(&file).unwrap_or(()); // index removal
    commit("main delete".to_string()).unwrap();

    // Go to alt and edit
    checkout("alt".to_string()).unwrap();
    commit_helper(&file, "alt edit", "alt edit");

    // Back to main and merge - should produce conflict (delete vs edit)
    checkout("main".to_string()).unwrap();
    let res = merge("alt".to_string());
    assert!(res.is_err()); // merge runs, but conflict entries produced
}

#[test]
fn merge_nested_file_conflict() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let f = root.join("src/core/lib.rs");
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    commit_helper(&f, "v1", "c1");

    // Create branch
    branch("work".to_string(), None).unwrap();

    // Main edits
    commit_helper(&f, "main edit", "main edit");

    // Go to work and make different edit
    checkout("work".to_string()).unwrap();
    commit_helper(&f, "work edit", "work edit");

    // Back to main and merge - should create conflict
    checkout("main".to_string()).unwrap();
    let res = merge("work".to_string());
    assert!(res.is_err()); // Runs but creates conflict
}

#[test]
fn merge_add_directory_one_side() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let f = root.join("x.txt");
    commit_helper(&f, "base", "c1");

    // Create branch
    branch("side".to_string(), None).unwrap();

    // Main adds entire directory
    let nd = root.join("docs/intro.txt");
    fs::create_dir_all(nd.parent().unwrap()).unwrap();
    commit_helper(&nd, "intro", "add docs");

    // Merge side (old state) - should be up-to-date
    let res = merge("side".to_string());
    assert!(res.is_ok());
}

#[test]
fn merge_both_add_dirs() {
    let tmp = TempDir::new().unwrap();
    let root = init_test_repo(&tmp);

    let base = root.join("b.txt");
    commit_helper(&base, "b", "c1");

    // Create branch
    branch("other".to_string(), None).unwrap();

    // Main adds dir A
    let d1 = root.join("a/a1.txt");
    fs::create_dir_all(d1.parent().unwrap()).unwrap();
    commit_helper(&d1, "aaa", "add a1");

    // Go to other and add dir B
    checkout("other".to_string()).unwrap();
    let d2 = root.join("b/b1.txt");
    fs::create_dir_all(d2.parent().unwrap()).unwrap();
    commit_helper(&d2, "bbb", "add b1");

    // Back to main and merge - should succeed (no conflicts, just combine both dirs)
    checkout("main".to_string()).unwrap();
    let res = merge("other".to_string());
    assert!(res.is_ok());
}
