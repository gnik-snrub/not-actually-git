use crate::core::{
    diff::{ get_all_diffs, DiffType },
};

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
        buf_str.push_str("\n\x1b[1;31mUntracked files:\x1b[0m\n");
        for path in untracked {
            buf_str.push_str(&format!("\t\x1b[31m? {}\x1b[0m\n", path));
        }
    }

    if !deleted.is_empty() || !modified.is_empty() {
        buf_str.push_str("\n\x1b[1;60mUnstaged:\x1b[0m");
    }

    if !deleted.is_empty() {
        buf_str.push_str("\n\t\x1b[1;35mDeleted:\x1b[0m\n");
        for path in deleted {
            buf_str.push_str(&format!("\t\x1b[35m- {}\x1b[0m\n", path));
        }
    }
    if !modified.is_empty() {
        buf_str.push_str("\n\t\x1b[1;33mModified:\x1b[0m\n");
        for path in modified {
            buf_str.push_str(&format!("\t\x1b[33m~ {}\x1b[0m\n", path));
        }
    }

    let staged_count = staged.len() + staged_delete.len() + added.len();

    if staged_count > 0 {
        buf_str.push_str("\n\n\x1b[1;60mStaged:\x1b[0m");
    }

    if !added.is_empty() {
        buf_str.push_str("\n\t\x1b[1;32mAdded Files:\x1b[0m\n");
        for path in added {
            buf_str.push_str(&format!("\t\x1b[32m+ {}\x1b[0m\n", path));
        }
    }

    if !staged.is_empty() {
        buf_str.push_str("\n\t\x1b[1;36mModified Files:\x1b[0m\n");
        for path in staged {
            buf_str.push_str(&format!("\t\x1b[36m~ {}\x1b[0m\n", path));
        }
    }

    if !staged_delete.is_empty() {
        buf_str.push_str("\n\t\x1b[1;34mDeleted Files:\x1b[0m\n");
        for path in staged_delete {
            buf_str.push_str(&format!("\t\x1b[34m- {}\x1b[0m\n", path));
        }
    }

    if print {
        println!("{buf_str}");
    }

    Ok(buf_str)
}
