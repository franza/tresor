extern crate base64;

use std::fmt;

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    EncryptionError,
    DecryptionError,
    KeyError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EncryptionError => f.write_str("Encryption error"),
            Error::DecryptionError => f.write_str("Decryption error"),
            Error::KeyError(message) => f.write_str(format!("Invalid key error: {}", message).as_str()),
        }
    }
}

pub fn encrypt(secret: &str, salt: &str, plaintext: &str) -> Result<String, Error> {
    let secret = align(secret, KEY_MASK, false)?;
    let key = Key::from_slice(secret.as_bytes());
    let cipher = Aes256Gcm::new(key);

    let aligned_salt = align(salt, NONCE_MASK, true)?;
    let nonce = Nonce::from_slice(aligned_salt.as_bytes());

    cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map(base64::encode)
        .map_err(|_| Error::EncryptionError)
}

pub fn decrypt(secret: &str, salt: &str, ciphertext: &str) -> Result<String, Error> {
    let secret = align(secret, KEY_MASK, false)?;
    let key = Key::from_slice(secret.as_bytes());
    let cipher = Aes256Gcm::new(key);

    let aligned_salt = align(salt, NONCE_MASK, true)?;
    let nonce = Nonce::from_slice(aligned_salt.as_bytes());

    fn dec_err<T, R>(_: T) -> Result<R, Error> { Err(Error::DecryptionError) }

    let decoded = base64::decode(ciphertext).or_else(dec_err)?;
    let decoded = decoded.as_slice();
    let plain_bytes = cipher.decrypt(nonce, decoded).or_else(dec_err)?;
    String::from_utf8(plain_bytes).or_else(dec_err)
}

const NONCE_MASK: &str = "111111111111";
const KEY_MASK: &str = "11111111111111111111111111111111";

fn align(val: &str, mask: &str, truncate: bool) -> Result<String, Error> {
    match val.len().cmp(&mask.len()) {
        Ordering::Equal => Ok(val.to_string()),
        Ordering::Less => {
            let (_, rest) = mask.split_at(val.len());
            Ok(format!("{}{}", val, rest))
        },
        Ordering::Greater if truncate => {
            let mut result = String::from(val);
            result.truncate(mask.len());
            Ok(result)
        },
        Ordering::Greater => {
            let message = format!("length must be less than {} characters", mask.len());
            Err(Error::KeyError(message))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{encrypt, decrypt, Error};
    use std::matches;

    #[test]
    fn encrypt_works() {
        let secret = "12345";
        let nonce = "randomstring";
        let value = "data to test";

        let result = encrypt(secret, nonce, value);

        assert!(matches!(result, Ok(_)));
        assert_ne!(result.unwrap().len(), 0);
    }

    #[test]
    fn decrypt_works() -> Result<(), Error> {
        let secret = "12345";
        let nonce = "randomstring";
        let value = "data to test";

        let ciphertext = encrypt(secret, nonce, value).unwrap();
        let result = decrypt(secret, nonce, &ciphertext)?;

        assert_eq!(value, result);
        Ok(())
    }
}