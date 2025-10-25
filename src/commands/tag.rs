use crate::core::refs::{
    list_refs,
};
use crate::core::repo::find_repo_root;

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

