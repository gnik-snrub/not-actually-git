use std::path::Path;

use crate::core::repo::find_repo_root;

pub fn read_index() -> std::io::Result<Vec<(String, String)>> {
    let index = find_repo_root()?.join(".nag").join("index");

    if !index.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Index not found",
        ))
    }

    let mut entries = vec![];

    let index_string = std::fs::read_to_string(index)?;
    for oid_and_path in index_string.lines() {
        if let Some((oid, path)) = oid_and_path.split_once("\t") {
            entries.push((oid.to_string(), path.to_string()));
        };
    }
    Ok(entries)
}

pub fn write_index(root: &Path, entries: &[(String, String)]) -> std::io::Result<()> {
    let index = find_repo_root()?.join(".nag").join("index");
    let mut buf = String::new();

    for (oid, path) in entries {
        buf.push_str(oid);
        buf.push('\t');
        buf.push_str(path);
        buf.push('\n');
    }

    Ok(())
}

