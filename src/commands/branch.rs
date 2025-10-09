use crate::core::io::{ read_file, write_file };
use crate::core::repo::find_repo_root;

pub fn branch(branch: String) -> std::io::Result<()> {
    let nag_head = find_repo_root()?.join(".nag");

    let refs_dir = nag_head.join("refs/heads");
    if refs_dir.join(&branch).exists() {
        println!("Branch {} already exists", branch);
        return Ok(());
    }

    let proj_head = nag_head.join("HEAD");
    let proj_head_contents = read_file(&proj_head.to_string_lossy());
    let head_str = String::from_utf8_lossy(&proj_head_contents);

    let target = head_str.trim();
    let branch_path_fragment = target.strip_prefix("ref: ").unwrap_or(target);
    let branch_path = nag_head.join(branch_path_fragment);
    let branch_contents = read_file(&branch_path.to_string_lossy());
    let oid = String::from_utf8_lossy(&branch_contents).trim().to_string();

    write_file(&oid.as_bytes().to_vec(), &refs_dir.join(&branch))?;

    println!("Branch {} created at {}", branch, oid);

    Ok(())
}
