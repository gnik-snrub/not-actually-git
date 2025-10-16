use crate::core::{
    io::read_file,
    hash::hash,
    diff::{ get_all_diffs, DiffType },
};

use std::fs::read_dir;
use std::path::Path;

pub fn status(print: bool) -> std::io::Result<String> {
    let diffs = get_all_diffs()?;
    let empty = vec![];

    let added = diffs.get(&DiffType::Added).unwrap_or(&empty);
    let modified = diffs.get(&DiffType::Modified).unwrap_or(&empty);
    let deleted = diffs.get(&DiffType::Deleted).unwrap_or(&empty);
    let untracked = diffs.get(&DiffType::Untracked).unwrap_or(&empty);
    let staged = diffs.get(&DiffType::Staged).unwrap_or(&empty);
    let staged_delete = diffs.get(&DiffType::StagedDelete).unwrap_or(&empty);

    let mut buf_str = String::new();

    if !untracked.is_empty() {
        buf_str.push_str("\nUntracked files:\n");
        for path in untracked {
            buf_str.push_str(&format!("\t{}\n", path));
        }
    }

    if !deleted.is_empty() || !modified.is_empty() {
        buf_str.push_str("\nUnstaged:");
    }

    if !deleted.is_empty() {
        buf_str.push_str("\n\tDeleted:\n");
        for path in deleted {
            buf_str.push_str(&format!("\t\t{}\n", path));
        }
    }
    if !modified.is_empty() {
        buf_str.push_str("\n\tModified:\n");
        for path in modified {
            buf_str.push_str(&format!("\t\t{}\n", path));
        }
    }

    let staged_count = staged.len() + staged_delete.len() + added.len();

    if staged_count > 0 {
        buf_str.push_str("\n\nStaged:");
    }

    if !added.is_empty() {
        buf_str.push_str("\n\tAdded Files:\n");
        for path in added {
            buf_str.push_str(&format!("\t\t{}\n", path));
        }
    }

    if !staged.is_empty() {
        buf_str.push_str("\n\tModified Files:\n");
        for path in staged {
            buf_str.push_str(&format!("\t\t{}\n", path));
        }
    }

    if !staged_delete.is_empty() {
        buf_str.push_str("\n\tDeleted Files:\n");
        for path in staged_delete {
            buf_str.push_str(&format!("\t\t{}\n", path));
        }
    }

    if print {
        println!("{buf_str}");
    }

    Ok(buf_str)
}
