use crate::core::refs::{
    list_refs,
    read_ref,
};
use crate::core::repo::find_repo_root;

use std::fs::remove_file;

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
        remove_file(path);
    }

    Ok(())
}
