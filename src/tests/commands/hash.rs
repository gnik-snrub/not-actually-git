use std::fs;
use std::io::Read;
use tempfile::TempDir;
use rand::random;

// Assuming you have these in your crate:
use crate::core::io::write_file;
use crate::commands::hash::hash;

fn setup_repo() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let objects = tmp.path().join(".nag").join("objects");
    fs::create_dir_all(&objects).unwrap();
    (tmp, objects)
}

#[test]
fn writes_new_blob_once() {
    let (_tmp, objects) = setup_repo();
    let data = b"hello".to_vec();
    let hash = hash(&data);
    write_file(data.clone(), &objects, &hash);

    let final_path = objects.join(format!("{hash}.blob"));
    assert!(final_path.exists());

    // Writing again with same content should not error or duplicate
    write_file(data, &objects, &hash);
    assert!(final_path.exists());
}

#[test]
fn different_content_different_hash() {
    let (_tmp, objects) = setup_repo();
    let h1 = hash(&b"hello".to_vec());
    let h2 = hash(&b"hello world".to_vec());

    write_file(b"hello".to_vec(), &objects, &h1);
    write_file(b"hello world".to_vec(), &objects, &h2);

    assert_ne!(h1, h2);
    assert!(objects.join(format!("{h1}.blob")).exists());
    assert!(objects.join(format!("{h2}.blob")).exists());
}

#[test]
fn file_contents_roundtrip() {
    let (_tmp, objects) = setup_repo();
    let bytes: Vec<u8> = (0..256).map(|_| random::<u8>()).collect();
    let hash = hash(&bytes);
    write_file(bytes.clone(), &objects, &hash);

    let path = objects.join(format!("{hash}.blob"));
    let mut f = fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    assert_eq!(bytes, buf);
}

#[test]
fn concurrent_writes_race_safe() {
    use std::sync::Arc;
    use std::thread;

    let (_tmp, objects) = setup_repo();
    let data = b"race test".to_vec();
    let hash = hash(&data);

    let objects_arc = Arc::new(objects);
    let mut handles = Vec::new();

    for _ in 0..10 {
        let objects = objects_arc.clone();
        let data = data.clone();
        let hash = hash.clone();
        handles.push(thread::spawn(move || {
            write_file(data, &objects, &hash);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_path = objects_arc.join(format!("{hash}.blob"));
    assert!(final_path.exists());

    // No stray temps
    let entries = fs::read_dir(objects_arc.as_path()).unwrap();
    for entry in entries {
        let name = entry.unwrap().file_name();
        let name = name.to_string_lossy();
        assert!(name.ends_with(".blob"), "unexpected leftover: {name}");
    }
}

#[test]
fn preexisting_object_dedupe() {
    let (_tmp, objects) = setup_repo();
    let data = b"existing".to_vec();
    let hash = hash(&data);
    let path = objects.join(format!("{hash}.blob"));

    // Manually create object file
    fs::write(&path, &data).unwrap();
    let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

    // Call write_file on same content
    write_file(data, &objects, &hash);

    let mtime_after = fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(mtime_before, mtime_after, "object should not be rewritten");
}
