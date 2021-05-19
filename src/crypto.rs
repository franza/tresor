extern crate base64;

use std::fmt;

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use std::cmp::Ordering;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error(err.to_string())
    }
}

impl From<aes_gcm::Error> for Error {
    fn from(err: aes_gcm::Error) -> Self {
        Error(err.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error(err.to_string())
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
        .map_err(Error::from)
}

pub fn decrypt(secret: &str, salt: &str, ciphertext: &str) -> Result<String, Error> {
    let secret = align(secret, KEY_MASK, false)?;
    let key = Key::from_slice(secret.as_bytes());
    let cipher = Aes256Gcm::new(key);

    let aligned_salt = align(salt, NONCE_MASK, true)?;
    let nonce = Nonce::from_slice(aligned_salt.as_bytes());

    let decoded = base64::decode(ciphertext)?;
    let decoded = decoded.as_slice();
    let plain_bytes = cipher.decrypt(nonce, decoded)?;
    String::from_utf8(plain_bytes).map_err(Error::from)
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
            Err(Error(message))
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