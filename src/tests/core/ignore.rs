use std::fs;
use std::path::Path;
use tempfile::TempDir;

use crate::core::ignore::should_ignore;

// Helper: initialize fake repo with .nag structure and cd into it
fn init_repo(tmp: &TempDir) -> std::path::PathBuf {
    let nag_root = tmp.path().join(".nag");
    let objects = nag_root.join("objects");
    fs::create_dir_all(&objects).unwrap();

    // important: pretend we’re “inside” the repo root
    std::env::set_current_dir(tmp.path()).unwrap();

    nag_root
}

// --- 1. Empty or missing ignore file ----------------------------------------
// Purpose: Ensure no ignores if .nagignore doesn’t exist
#[test]
fn should_ignore_returns_false_if_no_ignore_file() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);
    let path = tmp.path().join("foo.txt");
    fs::write(&path, "data").unwrap();

    assert!(!should_ignore(&path).unwrap());
}

// --- 2. Simple matching pattern ---------------------------------------------
// Purpose: Should ignore file matching "*.log"
#[test]
fn should_ignore_matches_simple_pattern() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    let ignore_path = tmp.path().join(".nagignore");
    fs::write(&ignore_path, "*.log\n").unwrap();

    let log_path = tmp.path().join("test.log");
    assert!(should_ignore(&log_path).unwrap());

    let txt_path = tmp.path().join("notes.txt");
    assert!(!should_ignore(&txt_path).unwrap());
}

// --- 3. Negation pattern ----------------------------------------------------
// Purpose: Ensure that later "!important.log" re-includes file
#[test]
fn should_ignore_honors_negation_patterns() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    fs::write(
        tmp.path().join(".nagignore"),
        "*.log\n!important.log\n",
    )
    .unwrap();

    let ignored_path = tmp.path().join("debug.log");
    let allowed_path = tmp.path().join("important.log");

    assert!(should_ignore(&ignored_path).unwrap());
    assert!(!should_ignore(&allowed_path).unwrap());
}

// --- 4. Directory pattern ---------------------------------------------------
// Purpose: Confirm entire directory ignores
#[test]
fn should_ignore_entire_directory() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    fs::write(tmp.path().join(".nagignore"), "build/\n").unwrap();
    fs::create_dir_all(tmp.path().join("build/output")).unwrap();

    let file_in_build = tmp.path().join("build/output/main.o");
    assert!(should_ignore(&file_in_build).unwrap());

    let file_elsewhere = tmp.path().join("src/main.rs");
    assert!(!should_ignore(&file_elsewhere).unwrap());
}

// --- 5. Invalid pattern -----------------------------------------------------
// Purpose: Should raise InvalidData error for malformed glob
#[test]
fn should_ignore_returns_error_on_invalid_pattern() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    fs::write(tmp.path().join(".nagignore"), "[]foo\n").unwrap();
    let path = tmp.path().join("foo.txt");

    let err = should_ignore(&path).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Invalid ignore pattern"));
}

// --- 6. Comment and blank line handling -------------------------------------
// Purpose: Ensure comments and blank lines are ignored
#[test]
fn should_ignore_skips_comments_and_blank_lines() {
    let tmp = TempDir::new().unwrap();
    init_repo(&tmp);

    fs::write(
        tmp.path().join(".nagignore"),
        "# comment line\n\n*.tmp\n",
    )
    .unwrap();

    let tmp_file = tmp.path().join("junk.tmp");
    let normal_file = tmp.path().join("data.txt");

    assert!(should_ignore(&tmp_file).unwrap());
    assert!(!should_ignore(&normal_file).unwrap());
}
