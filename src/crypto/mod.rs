mod error;

use openssl::rsa::{Rsa, Padding};
use openssl::symm::Cipher;
use openssl::error::ErrorStack;
use openssl::pkey::{Public, Private};
use std::ops::Add;
use std::fs;
use crate::crypto::error::CryptoError;
use std::io;

pub struct Crypto {
    priv_key: Rsa<Private>,
    publ_key: Rsa<Public>,
}

const PRIV_PEM_DIR: &str = "./pem/private.pem";
const PUBL_PEM_DIR: &str = "./pem/public.pem";

pub fn save_keys(crypto: &Crypto) -> std::io::Result<()> {
    fs::write(PRIV_PEM_DIR, crypto.priv_key.private_key_to_pem()?)
        .and_then(|_| fs::write(PUBL_PEM_DIR, crypto.publ_key.public_key_to_pem()?))
}

impl Crypto {
    pub fn init(passphrase: &str) -> Result<Crypto, ErrorStack> {
        let passphrase = passphrase.as_bytes();
        let rsa = Rsa::generate(2048)?;

        let priv_key_pem = rsa.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase)?;
        let publ_key_pem = rsa.public_key_to_pem()?;

        let crypto = Crypto {
            priv_key: Rsa::private_key_from_pem_passphrase(priv_key_pem.as_slice(), passphrase)?,
            publ_key: Rsa::public_key_from_pem(publ_key_pem.as_slice())?,
        };

        Ok(crypto)
    }

    pub fn from_fs(passphrase: &str) -> Result<Crypto, CryptoError> {
        let passphrase = passphrase.as_bytes();
        let priv_key_pem = fs::read(PRIV_PEM_DIR).map_err(CryptoError::from)?;
        let publ_key_pem = fs::read(PUBL_PEM_DIR).map_err(CryptoError::from)?;

        let crypto = Crypto {
            priv_key: Rsa::private_key_from_pem_passphrase(priv_key_pem.as_slice(), passphrase).map_err(CryptoError::from)?,
            publ_key: Rsa::public_key_from_pem(publ_key_pem.as_slice()).map_err(CryptoError::from)?,
        };

        Ok(crypto)
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, ErrorStack> {
        let mut buf: Vec<u8> = vec![0; self.publ_key.size() as usize];
        let _ = self.publ_key.public_encrypt(data, &mut buf, Padding::PKCS1)?;
        Ok(buf)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, ErrorStack> {
        let mut buf: Vec<u8> = vec![0; self.priv_key.size() as usize];
        let _ = self.priv_key.private_decrypt(&encrypted_data, &mut buf, Padding::PKCS1).unwrap();
        Ok(buf)
    }
}

