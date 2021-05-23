use tresor::crypto;
use tresor::storage;
use tresor::storage::{Error, Storage};
use tresor::storage::Entry;
use std::fmt;
use std::fmt::Formatter;

const DB_NAME: &str = "tresor.db";

// Initiates public and private keys and asks the passphrase
pub fn call_init() -> Result<(), ExecError> {
    storage::sqlite::setup(DB_NAME).map_err(ExecError::from).map(|_| ())
}

fn open_storage() -> Result<impl Storage, Error> {
    storage::sqlite::SqliteStorage::from_db(DB_NAME)
}

fn salt(entry: &Entry) -> String {
    entry.created_on.to_string()
}

fn encrypt_entry(password: &str, entry: Entry) -> Result<Entry, ExecError> {
    let mut new_entry = Entry::new(&entry.bucket, &entry.key, &entry.value);
    new_entry.value = crypto::encrypt(password, &salt(&new_entry), &entry.value)
        .map_err(|err| ExecError::EncryptionError(err.to_string()))?;
    Ok(new_entry)
}

fn decrypt_entry(password: &str, entry: &Entry) -> Result<String, ExecError> {
    crypto::decrypt(password, &salt(&entry), &entry.value)
        .map_err(|err| ExecError::DecryptionError(err.to_string()))
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecError {
    StorageAccessError(String),
    StorageOperationError(String),
    KeyNotFound { bucket: String, key: String },
    EncryptionError(String),
    DecryptionError(String),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            ExecError::StorageAccessError(_) =>
                format!(r#"Database access denied for "{}""#, DB_NAME),
            ExecError::StorageOperationError(_) =>
                format!("Failed to perform operation over local database"),
            ExecError::KeyNotFound { bucket, key } =>
                format!(r#"Key "{}" was not found in the bucket "{}""#, key, bucket),
            ExecError::EncryptionError(_) =>
                format!("Failed to encrypt the data"),
            ExecError::DecryptionError(_) =>
                format!("Failed to decrypt the data"),
        };
        f.write_str(message.as_str())
    }
}

impl From<storage::Error> for ExecError {
    fn from(err: storage::Error) -> Self {
        match err {
            Error::StorageAccessError(_) => ExecError::StorageAccessError(err.to_string()),
            _ => ExecError::StorageOperationError(err.to_string()),
        }
    }
}

// Stores the value in DB
pub fn call_store(bucket: &str, key: &str, value: &str) -> Result<(), ExecError> {
    let storage = open_storage()?;

    match storage.lookup(bucket, key)? {
        None => {
            let password = rpassword::prompt_password_stdout("Enter password: ").unwrap();
            let encrypted_entry = encrypt_entry(&password, Entry::new(bucket, key, value))?;
            storage.store(encrypted_entry)?;
        }
        Some(existing_entry) => {
            //Entry with this key already exists.
            //User must enter the password in order to overwrite the value
            let prompt = format!("Bucket {} already contains key {} with a value. \
                Enter the password which was used to encrypt it in order to overwrite: ", bucket, key);
            let password = rpassword::prompt_password_stdout(&prompt).unwrap();

            //Failing on purpose if decryption failed
            let _ = decrypt_entry(&password, &existing_entry)?;

            //Everything is okay, we ask for the password for the new value
            let password = rpassword::prompt_password_stdout("Enter new password: ").unwrap();
            let encrypted_entry = encrypt_entry(&password, Entry::new(bucket, key, value))?;
            storage.store(encrypted_entry)?;
        }
    };
    Ok(())
}

// Retrieves the value from safe storage and prints
pub fn call_get(bucket: &str, key: &str) -> Result<(), ExecError> {
    let storage = open_storage()?;
    let entry = storage.lookup(bucket, key)?
        .ok_or(ExecError::KeyNotFound { bucket: bucket.to_string(), key: key.to_string() })?;

    let password = rpassword::prompt_password_stdout("Enter password: ").unwrap();
    let value = decrypt_entry(&password, &entry)?;
    Ok(println!("{}", value))
}

// Removes the value from safe storage without printing it.
// Asks for a password which was used to encrypt the value before deleting it
pub fn call_delete(bucket: &str, key: &str) -> Result<(), ExecError> {
    let storage = open_storage()?;
    let entry = storage.lookup(bucket, key)?
        .ok_or(ExecError::KeyNotFound { bucket: bucket.to_string(), key: key.to_string() })?;

    let password = rpassword::prompt_password_stdout("Enter the password \
                which was used to encrypt this value: ").unwrap();

    let _ = decrypt_entry(&password, &entry)?;
    Ok(storage.delete(bucket, key)?)
}