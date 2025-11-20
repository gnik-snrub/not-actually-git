use std::path::Path;
use std::fs::read_dir;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::core::io::{ read_file, write_object };
use crate::core::index::{ read_index, write_index, IndexEntry, EntryType };
use crate::core::hash::hash;
use crate::core::repo::find_repo_root;
use crate::core::ignore::should_ignore;

pub fn add(path: &Path) -> std::io::Result<()> {
    let mut index = read_index()?;
    walk(path, &mut index)?;
    write_index(&index)?;
    Ok(())
}

fn walk(path: &Path, entries: &mut Vec<IndexEntry>) -> std::io::Result<()> {
    if !path.exists() {
        let repo_root = find_repo_root()?;
        let rel_path = path.strip_prefix(&repo_root).unwrap_or(path);
        let rel_str = rel_path.to_string_lossy().to_string();
        entries.retain(|entry| &entry.path != &rel_str);
        return Ok(())
    }
    if should_ignore(path)? {
        return Ok(());
    }
    if path.is_dir() {
        for child in read_dir(path)? {
            let dir = child.unwrap();
            walk(&dir.path(), entries)?;
        }
    } else if path.is_file() {
        let abs_path = path.canonicalize()?;

        let repo_root = find_repo_root()?;
        let rel_path = path.strip_prefix(&repo_root).unwrap_or(path);
        let mut rel_str = rel_path.to_string_lossy().to_string();

        rel_str = rel_str.replace('\\', "/");
        if rel_str.starts_with("./") {
            rel_str = rel_str[2..].to_string();
        }

        let file = read_file(&abs_path.to_string_lossy())?;
        let blob = hash(&file);
        write_object(&file, &blob)?;
        update_or_insert(blob, rel_str, entries)?;
    }
    Ok(())
}

fn update_or_insert(oid: String, path: String, entries: &mut Vec<IndexEntry>) -> std::io::Result<()> {
    let mut found = false;
    for entry in &mut *entries {
        if entry.path == path {
            entry.oids = vec![oid.clone()];     // overwrite OID always
            found = true;
            break;
        }
    }
    if !found {
        let real_path = Path::new(&path);
        let mode = if real_path.is_dir() {
            "040000".to_string()
        } else {
            #[cfg(unix)]
            {
                if real_path.metadata()?.permissions().mode() & 0o111 != 0 {
                    "100755".to_string()
                } else {
                    "100644".to_string()
                }
            }
            #[cfg(windows)]
            {
                "100644".to_string()
            }
        };

        let new_entry = IndexEntry {
            entry_type: EntryType::C,
            path: path,
            mode: mode,
            oids: vec![oid],
        };
        entries.push(new_entry);
    }

    Ok(())
}
