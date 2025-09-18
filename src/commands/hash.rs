use std::path::Path;
use sha2::{Sha256, Digest};
use crate::core::io::write_file;

pub fn hash(file_bytes: &Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    let result = hasher.finalize();
    let hex = format!("{:x}", result);

    let path = Path::new("./.nag/objects");
    let canon_path = match path.canonicalize() {
        Err(_e) => {
            println!("Error: creating blob file path");
            return hex;
        },
        Ok(p) => {
            p
        }
    };
    write_file(file_bytes.to_vec(), &canon_path, &hex);

    hex
}
