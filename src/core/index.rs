use crate::core::repo::find_repo_root;
use crate::core::io::write_file;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct IndexEntry {
    pub entry_type: EntryType,
    pub path: String,
    pub mode: String,
    pub oids: Vec<String>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum EntryType { C, X } // C is clean; X is conflicted
use std::fmt;

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::C => write!(f, "C"),
            EntryType::X => write!(f, "X"),
        }
    }
}


pub fn read_index() -> std::io::Result<Vec<IndexEntry>> {
    let index = find_repo_root()?.join(".nag").join("index");

    if !index.exists() {
        return Ok(Vec::new())
    }

    let mut entries = vec![];

    let index_string = std::fs::read_to_string(index)?;
    for line in index_string.lines() {
        let items = line.split('\t').collect::<Vec<&str>>();

        if items.len() < 4 {
            continue;
        }

        let entry_type = match items[0] {
            "C" => EntryType::C,
            "X" => EntryType::X,
            _ => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid entry type"))
            }
        };
        let mode = items[1];
        let path = items[2];
        let oids = items[3..].to_vec().iter().map(|x| x.to_string()).collect();
        let entry = IndexEntry {
            entry_type: entry_type,
            path: path.to_string(),
            mode: mode.to_string(),
            oids: oids,
        };
        entries.push(entry);
    }
    Ok(entries)
}

pub fn write_index(entries: &Vec<IndexEntry>) -> std::io::Result<()> {
    let index = find_repo_root()?.join(".nag").join("index");
    let mut buf = String::new();

    for entry in entries {
        match entry.entry_type {
            EntryType::C => buf.push_str("C\t"),
            EntryType::X => buf.push_str("X\t"),
        }
        buf.push_str(&entry.mode);
        buf.push('\t');
        buf.push_str(&entry.path);
        for oid in &entry.oids {
            buf.push('\t');
            buf.push_str(&oid);
        }
        buf.push('\n');
    }
    write_file(&buf.as_bytes().to_vec(), &index)?;

    Ok(())
}
