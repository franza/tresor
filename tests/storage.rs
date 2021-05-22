use tresor::storage::{Storage, Error};
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
    let result = storage.lookup("undefined", "undefined");

    assert!(matches!(result, Ok(None)));

    teardown();
}

#[test]
#[serial]
fn insert_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let result = storage.store("b", "k", "v");

    assert_eq!(result, Ok(()));
    teardown();
}

#[test]
#[serial]
fn lookup_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let bucket = "b";
    let key = "k";
    let value = "v";
    storage.store(bucket, key, value).unwrap();
    let result = storage.lookup(bucket, key);

    assert!(matches!(result, Ok(Some(_))));

    let entry = result.unwrap().unwrap();

    assert_eq!(entry.bucket, bucket.to_string());
    assert_eq!(entry.key, key.to_string());
    assert_eq!(entry.value, value.to_string());
    assert_eq!(entry.modified_on, None);

    teardown();
}

#[test]
#[serial]
fn update_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let bucket = "b";
    let key = "k";
    let value1 = "v1";
    storage.store(bucket, key, value1).unwrap();
    let entry = storage.lookup(bucket, key).unwrap().unwrap();

    assert_eq!(entry.modified_on, None);

    let value2 = "v2";
    storage.store(bucket, key, value2).unwrap();
    let entry = storage.lookup(bucket, key).unwrap().unwrap();

    assert_eq!(entry.value, value2.to_string());
    assert!(matches!(entry.modified_on, Some(_)));

    teardown();
}

#[test]
#[serial]
fn delete_non_existent_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let bucket = "b";
    let key = "k";
    let result = storage.delete(bucket, key);

    assert!(matches!(result, Ok(_)));
    teardown();
}

#[test]
#[serial]
fn delete_entry() {
    setup();

    let storage = sqlite::setup("test.db").unwrap();
    let bucket = "b";
    let key = "k";
    let value1 = "v1";
    storage.store(bucket, key, value1).unwrap();

    let result = storage.delete(bucket, key);

    assert!(matches!(result, Ok(_)));
    teardown();
}
