use crate::core::refs::{
    list_refs,
    read_ref,
    update_ref,
    resolve_head,
};
use crate::core::repo::find_repo_root;
use crate::core::hash::hash;
use crate::core::io::write_object;

use std::fs::remove_file;

pub fn tag(tag_name: Option<String>, commit: Option<String>, message: Option<String>) -> std::io::Result<()> {
    if let Some(name) = tag_name {
        let oid = if commit.is_none() {
            resolve_head()?.1
        } else if let Some(commit_oid) = commit {
            let commit_path = find_repo_root()?.join(".nag/objects").join(&commit_oid);
            if !commit_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Commit not found"),
                ));
            }
            commit_oid
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error in data found"),
            ));
        };
        if let Some(msg) = message {
            let mut annotated = String::new();
            annotated.push_str(&format!("object {}\n\n", oid));
            annotated.push_str(&msg);
            let bytes = annotated.as_bytes().to_vec();
            let annotated_tag_oid = hash(&bytes);
            write_object(&bytes, &annotated_tag_oid)?;
            update_ref(&format!("refs/tags/{}", name), &annotated_tag_oid)?;
        } else {
            update_ref(&format!("refs/tags/{}", name), &oid)?;
        }
    }

    Ok(())
}

pub fn list_tags(print: bool) -> std::io::Result<String> {
    let branches = list_refs("refs/tags")?;

    let mut output = String::new();
    for entry in &branches {
        output.push_str(&format!("{}\n", entry));
    }

    if print {
        println!("Found tags:");
        for entry in &branches {
            println!("\t{}", entry);
        }
    }

    Ok(output)
}

pub fn delete_tag(tag_name: String) -> std::io::Result<()> {
    if read_ref(&format!("refs/tags/{}", tag_name)).is_ok() {
        let path = find_repo_root()?.join(".nag/refs/tags").join(tag_name);
        remove_file(path)?;
    }

    Ok(())
}
