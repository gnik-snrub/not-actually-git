use std::fs;
use std::io::Read;
use tempfile::TempDir;
use rand::random;
use std::sync::Arc;
use std::thread;

use crate::core::io::write_object;
use crate::commands::hash::hash;
use crate::tests::common::setup_nag_repo;

#[test]
fn writes_new_blob_once() {
    let tmp = TempDir::new().unwrap();
    let objects = setup_nag_repo(&tmp);

    let data = b"hello".to_vec();
    let hash = hash(&data);

    write_object(&data, &hash).unwrap();

    let final_path = objects.join(&hash);
    assert!(final_path.exists());

    // writing same content twice is safe
    write_object(&data, &hash).unwrap();
    assert!(final_path.exists());
}

#[test]
fn different_content_different_hash() {
    let tmp = TempDir::new().unwrap();
    let objects = setup_nag_repo(&tmp);

    let d1 = b"hello".to_vec();
    let d2 = b"hello world".to_vec();
    let h1 = hash(&d1);
    let h2 = hash(&d2);

    write_object(&d1, &h1).unwrap();
    write_object(&d2, &h2).unwrap();

    assert_ne!(h1, h2);
    assert!(objects.join(&h1).exists());
    assert!(objects.join(&h2).exists());
}

#[test]
fn file_contents_roundtrip() {
    let tmp = TempDir::new().unwrap();
    setup_nag_repo(&tmp);

    let bytes: Vec<u8> = (0..256).map(|_| random::<u8>()).collect();
    let hash = hash(&bytes);

    write_object(&bytes, &hash).unwrap();

    let path = tmp.path().join(".nag").join("objects").join(&hash);
    let mut f = fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    assert_eq!(bytes, buf);
}

#[test]
fn concurrent_writes_race_safe() {
    let tmp = TempDir::new().unwrap();
    let objects = setup_nag_repo(&tmp);

    let data = b"race test".to_vec();
    let hash = hash(&data);

    let objects_arc = Arc::new(objects.clone());
    let mut handles = Vec::new();

    for _ in 0..10 {
        let data = data.clone();
        let hash = hash.clone();
        handles.push(thread::spawn(move || {
            write_object(&data, &hash).unwrap();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_path = objects_arc.join(&hash);
    assert!(final_path.exists());

    // only OID files, no temp leftovers
    for entry in fs::read_dir(objects_arc.as_path()).unwrap() {
        let name = entry.unwrap().file_name();
        let name = name.to_string_lossy();
        assert_eq!(name.len(), 64, "unexpected leftover: {name}");
    }
}

#[test]
fn preexisting_object_dedupe() {
    let tmp = TempDir::new().unwrap();
    let objects = setup_nag_repo(&tmp);

    let data = b"existing".to_vec();
    let hash = hash(&data);
    let path = objects.join(&hash);

    // manually create object file
    fs::write(&path, &data).unwrap();
    let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

    // call write_object on same content
    write_object(&data, &hash).unwrap();

    let mtime_after = fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(mtime_before, mtime_after, "object should not be rewritten");
}
