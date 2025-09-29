use std::fs;
use std::fs::DirEntry;
use std::path::{PathBuf, Path};
use std::os::unix::fs::PermissionsExt;
use std::collections::HashMap;
use crate::commands::hash::hash;
use crate::core::io::{ write_object, read_file };
use crate::core::repo::find_repo_root;

fn format_entry(perms: &str, name: &str, oid: &str) -> String {
    let mut entry = String::new();
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
                let data = read_file(&p.path().display().to_string());
                let blob = hash(&data);
                let entry = format_entry(perms, &name_str, &blob);
                string_buf.push_str(&entry);
            } else if p_type.is_dir() {
                let dir_path = write_tree(&p.path());
                match dir_path {
                    Ok(sub_dirs) => {
                        let entry = format_entry(perms, &name_str, &sub_dirs);
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

pub fn write_tree_from_index(index: &Vec<(String, String)>) -> std::io::Result<String> {
    let mut groups: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for entry in index {
        let oid_str = &entry.0;
        let path_str = &entry.1;
        let p = Path::new(&path_str);

        if let Some(first) = p.components().next() {
            let rest = p.strip_prefix(first.as_os_str()).unwrap_or(Path::new(""));
            if rest.as_os_str().is_empty() {
                groups.entry("".to_string())
                    .or_default()
                    .push((oid_str.clone(), first.as_os_str().to_string_lossy().to_string()));
            } else {
                groups.entry(first.as_os_str().to_string_lossy().to_string())
                    .or_default()
                    .push((oid_str.clone(), rest.to_string_lossy().to_string()));
            }
        }
    }

    let mut str_buf = String::new();

    if let Some(files) = groups.remove("") {
        let repo_root = find_repo_root()?; // project root
        let objects_dir = repo_root.join(".nag").join("objects");

        for (oid, path) in files {
            let obj_path = objects_dir.join(&oid);
            if !obj_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("missing blob object for {}", path),
                ));
            }

            // TODO Refactor Index to include permissions
            // TODO Include dynamic permissions from get_perms
            // ...I really should have done that from the start

            let entry = format_entry("100644", &path, &oid);
            str_buf.push_str(&entry);
        }
    }

    for group in groups.iter() {
        let sub_dir = write_tree_from_index(group.1)?;
        let entry = format_entry("040000", &group.0, &sub_dir);
        str_buf.push_str(&entry);
    }

    let buf_bytes: Vec<u8> = str_buf.as_bytes().to_vec();
    let tree_hash = hash(&buf_bytes);
    write_object(&buf_bytes, &tree_hash)?;
    Ok(tree_hash)
}
