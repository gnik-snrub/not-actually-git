use crate::core::hash::hash;
use crate::core::io::{ read_file, write_object };
use crate::core::index::{ read_index, write_index, EntryType };

pub fn resolve(path: &str) -> std::io::Result<()> {
    let mut index = read_index()?;

    let entry = index.iter_mut().find(|e| e.path == path);
    if let Some(entry) = entry {

        let file_bytes = read_file(&path.to_string())?;
        let blob = hash(&file_bytes);
        write_object(&file_bytes, &blob)?;

        entry.oids = vec![blob.clone()];
        entry.entry_type = EntryType::C;
        write_index(&index)?;

        return Ok(());
    }

    return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("Invalid path: {}", path),
    ));
}
