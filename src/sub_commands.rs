use crate::crypto::*;

// Initiates public and private keys and asks the passphrase
pub fn call_init() {
    println!("Called init");
    let passphrase = "123";
    let cry = Crypto::init(passphrase).unwrap();

    let data = "secret data";
    let encrypted = cry.encrypt(data.as_bytes()).unwrap();

    println!("encrypted {:?}", encrypted);

    let decrypted = cry.decrypt(encrypted.as_slice()).unwrap();
    println!("decrypted {}", String::from_utf8(Vec::from(decrypted)).unwrap())
}

// Stores the value in DB
pub fn call_store(bucket: &str, key: &str, value: &str) {
    println!("Called store with bucket {}, key {} and value {}", bucket, key, value)
}

// Retrieves the value from safe storage and prints
pub fn call_get(bucket: &str, key: &str) {
    println!("Called get with bucket {}, key {} and value", bucket, key)
}

// Removes the value from safe storage without printing it
pub fn call_delete(bucket: &str, key: &str) {
    println!("Called delete with bucket {}, key {} and value", bucket, key)
}