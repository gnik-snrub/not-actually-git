use std::fs;
use std::fs::DirEntry;
use std::path::{PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::collections::HashMap;

use crate::core::hash::hash;
use crate::core::io::{ write_object, read_file };
use crate::core::repo::find_repo_root;
use crate::core::index::{ IndexEntry, EntryType };

fn format_entry(entry_type: &EntryType, perms: &str, name: &str, oid: &str) -> String {
    let mut entry = String::new();
    entry.push_str(&entry_type.to_string());
    entry.push('\t');
    entry.push_str(perms);
    entry.push('\t');
    entry.push_str(name);
    entry.push('\t');
    entry.push_str(oid);
    entry.push('\n');
    entry
}

fn get_perms(entry: &DirEntry) -> &str {
    let metadata = entry.metadata();
    match metadata {
        Err(e) => {
            println!("Error: {:?}", e);
            return "";
        },
        Ok(meta) => {
            if meta.is_dir() {
                return "040000";
            }
            else if meta.is_file() {
                let mode = meta.permissions().mode();
                if mode & 0o111 != 0 {
                    return "100755";
                } else {
                    return "100644";
                }
            } else if meta.file_type().is_symlink() {
                return "120000";
            }
        }
    }
    return "";
}

pub fn write_tree(root_path: &PathBuf) -> std::io::Result<String> {
    let mut string_buf = String::new();
    let paths = fs::read_dir(root_path);
    for path in paths? {
        if let Ok(p) = path {
            let perms = get_perms(&p);
            let name = p.file_name();
            let name_str = name.to_string_lossy();
            if name == ".nag" {
                continue;
            }
            let p_type = p.file_type()?;
            if p_type.is_file() {
                let data = read_file(&p.path().display().to_string())?;
                let blob = hash(&data);
                let entry = format_entry(&EntryType::C, perms, &name_str, &blob);
                string_buf.push_str(&entry);
            } else if p_type.is_dir() {
                let dir_path = write_tree(&p.path());
                match dir_path {
                    Ok(sub_dirs) => {
                        let entry = format_entry(&EntryType::C, perms, &name_str, &sub_dirs);
                        string_buf.push_str(&entry);
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
    }
    let buffer_bytes = string_buf.into_bytes();
    let tree_hash = hash(&buffer_bytes);
    write_object(&buffer_bytes, &tree_hash)?;
    Ok(tree_hash)
}

pub fn write_tree_from_index(index: &Vec<IndexEntry>) -> std::io::Result<String> {
    let mut groups: HashMap<String, Vec<IndexEntry>> = HashMap::new();

    let filtered_index = index.iter().filter(|entry| entry.entry_type == EntryType::C).collect::<Vec<&IndexEntry>>();

    for entry in filtered_index {
        let oid = &entry.oids[0];

        if let Some((first, rest)) = entry.path.split_once('/') {
            let e = IndexEntry {
                entry_type: entry.entry_type.clone(),
                path: rest.to_string(),
                mode: entry.mode.clone(),
                oids: vec![oid.clone()],
            };
            groups.entry(first.to_string())
                .or_default()
                .push(e);
        } else {
            let e = IndexEntry {
                entry_type: entry.entry_type.clone(),
                path: entry.path.to_string(),
                mode: entry.mode.clone(),
                oids: vec![oid.clone()],
            };
            groups.entry("".to_string())
                .or_default()
                .push(e);
        }
    }

    let mut str_buf = String::new();

    if let Some(files) = groups.remove("") {
        let repo_root = find_repo_root()?; // project root
        let objects_dir = repo_root.join(".nag").join("objects");

        for item in files {
            let obj_path = objects_dir.join(&item.oids[0]);
            if !obj_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("missing blob object for {}", &item.path),
                ));
            }

            let entry = format_entry(&item.entry_type, item.mode.as_str(), &item.path, &item.oids[0]);
            str_buf.push_str(&entry);
        }
    }

    for group in groups.iter() {
        let sub_dir = write_tree_from_index(group.1)?;
        let entry = format_entry(&EntryType::C, "040000", &group.0, &sub_dir);
        str_buf.push_str(&entry);
    }

    let buf_bytes: Vec<u8> = str_buf.as_bytes().to_vec();
    let tree_hash = hash(&buf_bytes);
    write_object(&buf_bytes, &tree_hash)?;
    Ok(tree_hash)
}

pub fn read_tree_to_index(tree_oid: &str) -> std::io::Result<Vec<IndexEntry>> {
    let mut entries = vec![];

    read_t_to_i_walk(tree_oid, &mut entries)?;

    Ok(entries)
}

fn read_t_to_i_walk(tree_oid: &str, entries: &mut Vec<IndexEntry>) -> std::io::Result<()> {
    let tree_str = find_repo_root()?.join(".nag").join("objects").join(tree_oid);
    let tree_bytes = read_file(&tree_str.to_string_lossy())?;
    let tree_str = String::from_utf8_lossy(&tree_bytes);

    for line in tree_str.lines() {
        let parts: Vec<&str> = line.split('\t').collect();

        match parts[0] {
            "C" => {
                match parts[1] {
                    "100644" | "100755" => { // regular file / exec file
                        let path = parts[2].to_string();
                        let oid = parts[3].to_string();
                        entries.push(IndexEntry {
                            entry_type: EntryType::C,
                            path: path,
                            mode: parts[1].to_string(),
                            oids: vec![oid],
                        });
                    }
                    "040000" => { // directory
                        let dirname = parts[2];
                        let subtree_oid = parts[3];
                        let mut subtree_entries = vec![];
                        read_t_to_i_walk(subtree_oid, &mut subtree_entries)?;
                        for mut entry in subtree_entries {
                            entry.path = format!("{}/{}", dirname, entry.path);
                            entries.push(entry);
                        }
                    }
                    _ => {
                        /*
                        Can't decide to return an error or just ignore on symlinks
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid entry mode",
                        ));
                        */
                    }
                }
            },
            "X" => {
                // Conflicted entries are ignored for now
            },
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid entry type",
                ));
            },
        }
    }

    Ok(())
}
