use std::io;

use crate::crypto;

// Initiates public and private keys and asks the passphrase
pub fn call_init() {
    
}

// Stores the value in DB
pub fn call_store(bucket: &str, key: &str, value: &str) {
    let secret = {
        let mut secret = String::new();
        println!("Enter the password");
        io::stdin().read_line(&mut secret).expect("Failed to read the password");
        secret
    };

    // use it as nonce - not sure if it's legal
    let nonce = format!("{}_{}", bucket, key);

    let ciphertext = crypto::encrypt(&secret, &nonce, &value).unwrap();
    print!("Encrypted {} to {}", value, ciphertext);
}

// Retrieves the value from safe storage and prints
pub fn call_get(bucket: &str, key: &str) {
    println!("Called get with bucket {}, key {} and value", bucket, key)
}

// Removes the value from safe storage without printing it
pub fn call_delete(bucket: &str, key: &str) {
    println!("Called delete with bucket {}, key {} and value", bucket, key)
}