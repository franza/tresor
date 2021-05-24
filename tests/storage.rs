use tresor::storage::{Storage, Error, Entry};
use tresor::storage::sqlite;
use tresor::storage::sqlite::*;

#[macro_use]
extern crate serial_test;

fn setup() {
    reset("test.db").or::<Error>(Ok(())).unwrap();
}

fn teardown() {
    reset("test.db").or::<Error>(Ok(())).unwrap();
}

#[test]
#[serial]
fn lookup_nothing() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let result = storage.lookup_entry("undefined", "undefined");

    assert!(matches!(result, Ok(None)));

    teardown();
}

#[test]
#[serial]
fn insert_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let result = storage.store_entry(Entry::new("b", "k", "v"));

    assert_eq!(result, Ok(()));
    teardown();
}

#[test]
#[serial]
fn lookup_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let stored_entry = Entry::new("b", "k", "v");
    storage.store_entry(stored_entry).unwrap();
    let result = storage.lookup_entry("b", "k");

    assert!(matches!(result, Ok(Some(_))));

    let entry = result.unwrap().unwrap();

    assert_eq!(entry.bucket, "b");
    assert_eq!(entry.key, "k");
    assert_eq!(entry.value, "v");

    teardown();
}

#[test]
#[serial]
fn update_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let entry1 = Entry::new("b", "k", "v1");
    storage.store_entry(entry1).unwrap();

    let entry2 = Entry::new("b", "k", "v2");
    let created_on = entry2.created_on;
    storage.store_entry(entry2).unwrap();
    let entry = storage.lookup_entry("b", "k").unwrap().unwrap();

    assert_eq!(entry.value, "v2".to_string());
    assert_eq!(entry.created_on, created_on);

    teardown();
}

#[test]
#[serial]
fn delete_non_existent_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let bucket = "b";
    let key = "k";
    let result = storage.delete_entry(bucket, key);

    assert!(matches!(result, Ok(_)));
    teardown();
}

#[test]
#[serial]
fn delete_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    storage.store_entry(Entry::new("b", "k", "v1")).unwrap();

    let result = storage.delete_entry("b", "k");

    assert!(matches!(result, Ok(_)));
    teardown();
}

#[test]
#[serial]
fn list_buckets() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();

    let buckets = storage.list_buckets();
    assert_eq!(buckets, Ok(Vec::new()));

    storage.store_entry(Entry::new("b1", "k1", "v1")).unwrap();
    storage.store_entry(Entry::new("b2", "k2", "v2")).unwrap();
    storage.store_entry(Entry::new("b3", "k3", "v3")).unwrap();

    let buckets = storage.list_buckets();
    assert_eq!(buckets, Ok(vec!["b1".into(), "b2".into(), "b3".into()]));

    teardown();
}

#[test]
#[serial]
fn list_entries() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();

    storage.store_entry(Entry::new("b1", "k1", "v1")).unwrap();
    storage.store_entry(Entry::new("b1", "k2", "v2")).unwrap();
    storage.store_entry(Entry::new("b1", "k3", "v3")).unwrap();

    let entries = storage.list_entries("b1");
    assert_eq!(entries, Ok(vec![
        Entry::new("b1", "k1", "v1"),
        Entry::new("b1", "k2", "v2"),
        Entry::new("b1", "k3", "v3"),
    ]));

    let entries = storage.list_entries("notexist");
    assert_eq!(entries, Ok(Vec::new()));

    teardown();
}
