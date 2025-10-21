use crate::core::repo::find_repo_root;
use crate::core::io::{ read_file, write_file };

use std::path::Path;
use std::fs::read_dir;

pub fn resolve_head() -> std::io::Result<(Option<String>, String)> {
    let nag_head = find_repo_root()?.join(".nag");
    let proj_head = nag_head.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy())?;
    let head_str = String::from_utf8_lossy(&proj_head_contents);
    let trimmed_head = head_str.trim();

    println!("head: {:?}", trimmed_head);

    if trimmed_head.starts_with("ref: ") {
        // HEAD is pointing to a branch
        let branch_path = nag_head.join(trimmed_head.strip_prefix("ref: ").unwrap_or(trimmed_head));
        let branch_path_str = branch_path.to_string_lossy().to_string();
        let branch_contents = read_file(&branch_path_str)?;
        let commit_oid = String::from_utf8_lossy(&branch_contents);
        let trimmed_oid = commit_oid.trim();

        let branch_name = branch_path.file_name().unwrap();

        return Ok((Some(branch_name.to_string_lossy().to_string()), trimmed_oid.to_string()))
    } else {
        // HEAD is detached, and directly contains a commit oid
        return Ok((None, trimmed_head.to_string()))
    }
}

pub fn read_ref(ref_name: &str) -> std::io::Result<String> {
    let ref_name_full = if !ref_name.starts_with("refs/") {
        format!("refs/heads/{}", ref_name)
    } else {
        ref_name.to_string()
    };

    let ref_path = find_repo_root()?.join(".nag").join(ref_name_full);
    let ref_contents = read_file(&ref_path.to_string_lossy())?;
    let ref_str = String::from_utf8_lossy(&ref_contents);
    let trimmed = ref_str.trim().to_string();

    Ok(trimmed)
}

pub fn update_ref(name: &str, oid: &str) -> std::io::Result<()> {
    let name_full = if !name.starts_with("refs/") {
        format!("refs/heads/{}", name)
    } else {
        name.to_string()
    };

    let full_path = find_repo_root()?.join(".nag").join(name_full);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    write_file(&oid.as_bytes().to_vec(), &full_path)?;

    Ok(())
}

pub fn set_head_ref(branch: &str) -> std::io::Result<()> {
    if read_ref(branch).is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Branch '{}' not found", branch),
        ));
    }

    let nag_dir = find_repo_root()?.join(".nag");
    let head_path = nag_dir.join("HEAD");
    let ref_line = format!("ref: refs/heads/{}\n", branch);
    write_file(&ref_line.as_bytes().to_vec(), &head_path)?;

    Ok(())
}

pub fn set_head_detached(oid: &str) -> std::io::Result<()> {
    let nag_dir = find_repo_root()?.join(".nag");
    let head_path = nag_dir.join("HEAD");
    let object_path = nag_dir.join("objects").join(oid);

    if read_file(&object_path.as_os_str().to_string_lossy().to_string()).is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Commit object '{}' not found", object_path.display()),
        ));
    }

    write_file(&oid.as_bytes().to_vec(), &head_path)?;

    Ok(())
}

pub fn list_refs(prefix: &str) -> std::io::Result<Vec<String>> {
    let nag_head = find_repo_root()?.join(".nag");
    let refs_dir = nag_head.join(prefix);

    let mut all_refs = Vec::new();
    collect_refs(&refs_dir, &mut all_refs, Some(String::new()))?;

    all_refs.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    Ok(all_refs)
}


fn collect_refs(path: &Path, refs: &mut Vec<String>, prefix: Option<String>) -> std::io::Result<()> {
    if path.is_dir() {
        for child in read_dir(path)? {
            let dir = child.unwrap();
            let name = dir.file_name().to_string_lossy().to_string();

            let full_name = match &prefix {
                Some(pre) if !pre.is_empty() => format!("{}/{}", pre, name),
                _ => name.clone(),
            };

            if dir.path().is_dir() {
                collect_refs(&dir.path(), refs, Some(full_name))?;
            } else if dir.path().is_file() {
                refs.push(full_name);
            }
        }
    }
    Ok(())
}
