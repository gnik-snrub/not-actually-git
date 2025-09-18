use std::fs::{File, remove_file};
use std::io::Write;
use std::path::Path;
use rand::random;

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

pub fn write_file(file: Vec<u8>, path: &Path, hash: &String) {
    let final_path = path.join(hash).with_extension("blob");

    if final_path.exists() {
        return;
    }

    let process = std::process::id().to_string();
    let random = random::<u64>().to_string();

    let temp_path = final_path.with_extension(format!("tmp.{process}.{random}"));
    let mut temp_file = File::create(&temp_path).unwrap();
    let _ = temp_file.write(&file);
    let _ = temp_file.sync_all();

    let rename = std::fs::rename(&temp_path, final_path);

    match rename {
        Ok(()) => {},
        Err(e) => {
            println!("Error renaming blob: {e}");
            let _ = remove_file(temp_path);
            return;
        }
    }

    let dir = File::open(path).unwrap();
    let _ = dir.sync_all();
}
