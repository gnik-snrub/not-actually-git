use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use crate::commands::hash::hash;
use crate::core::io::{ write_object, read_file };
use crate::core::repo::find_repo_root;

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
