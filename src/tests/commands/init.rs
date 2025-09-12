// src/tests/init_tests.rs
use std::{fs, env, path::PathBuf, io};
use tempfile::TempDir;

// Adjust this path to your actual module path:
use crate::commands::init::init;

fn read_to_string(p: impl Into<PathBuf>) -> io::Result<String> {
    fs::read_to_string(p.into())
}

#[test]
fn init_creates_repo_structure_in_cwd() {
    let tmp = TempDir::new().unwrap();
    let old = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    init(None);

    assert!(tmp.path().join(".nag/objects").is_dir(), "objects/");
    assert!(tmp.path().join(".nag/refs/heads").is_dir(), "refs/heads/");
    assert!(tmp.path().join(".nag/HEAD").is_file(), "HEAD");

    env::set_current_dir(old).unwrap();
}

#[test]
fn head_points_to_main() {
    let tmp = TempDir::new().unwrap();
    let old = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    init(None);

    let head = read_to_string(".nag/HEAD").unwrap();
    assert_eq!(head.trim(), "ref: refs/heads/main");

    env::set_current_dir(old).unwrap();
}

#[test]
fn bootstrap_branch_file_exists() {
    let tmp = TempDir::new().unwrap();
    let old = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    init(None);

    assert!(PathBuf::from(".nag/refs/heads/main").is_file());

    env::set_current_dir(old).unwrap();
}

#[test]
fn idempotent_double_init() {
    let tmp = TempDir::new().unwrap();
    let old = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    init(None);
    init(None); // should not panic or rewrite structure destructively

    assert!(PathBuf::from(".nag/objects").is_dir());
    assert!(PathBuf::from(".nag/refs/heads/main").is_file());

    env::set_current_dir(old).unwrap();
}

#[test]
fn init_with_explicit_path_some() {
    let tmp = TempDir::new().unwrap();
    let repo_path = tmp.path().to_path_buf().display().to_string();

    init(Some(repo_path.clone()));

    assert!(tmp.path().join(".nag/objects").is_dir());
    assert!(tmp.path().join(".nag/refs/heads/main").is_file());

    let head = read_to_string(tmp.path().join(".nag/HEAD")).unwrap();
    assert_eq!(head.trim(), "ref: refs/heads/main");
}

