use std::fs::{File, remove_file};
use std::io::Write;
use std::path::Path;
use rand::random;
use crate::core::repo::find_repo_root;

pub fn read_file(path: &str) -> Vec<u8> {
    match std::fs::read(path) {
        Ok(bytes) => {
            bytes
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            vec![]
        }
    }
}

pub fn write_file(file: &Vec<u8>, path: &Path) -> std::io::Result<()>  {
    let final_path = path.to_path_buf();

    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if final_path.exists() {
        return Ok(());
    }

    let process = std::process::id().to_string();
    let random = random::<u64>().to_string();

    let temp_path = final_path.with_extension(format!("tmp.{process}.{random}"));
    let mut temp_file = File::create(&temp_path).unwrap();
    temp_file.write(&file)?;
    temp_file.sync_all()?;

    let rename = std::fs::rename(&temp_path, &final_path);

    match rename {
        Ok(()) => {},
        Err(e) => {
            println!("Error renaming blob: {e}");
            remove_file(temp_path)?;
            ()
        }
    }

    if let Some(parent) = final_path.parent() {
        let dir_file = File::open(parent)?;
        dir_file.sync_all()?;
    }
    Ok(())
}

pub fn write_object(data: &Vec<u8>, oid: &String) -> std::io::Result<()> {
    let obj_path = find_repo_root()?.join(".nag").join("objects").join(oid);
    write_file(data, &obj_path)?;

    Ok(())
}
