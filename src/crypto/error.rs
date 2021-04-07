use std::error::Error;
use std::{fmt, io};
use std::fmt::Formatter;
use openssl::error::ErrorStack;
use crate::crypto::error::InnerError::{OpenSSLError, IOError};

#[derive(Debug)]
enum InnerError {
    OpenSSLError(ErrorStack),
    IOError(io::Error),
}

impl InnerError {
    fn original(&self) -> &dyn Error {
        match self {
            InnerError::OpenSSLError(e) => e,
            InnerError::IOError(e) => e,
        }
    }
}

#[derive(Debug)]
pub struct CryptoError {
    inner: InnerError,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.original())
    }
}

impl Error for CryptoError {

}

impl From<ErrorStack> for CryptoError {
    fn from(e: ErrorStack) -> Self {
        CryptoError { inner: OpenSSLError(e) }
    }
}

impl From<io::Error> for CryptoError {
    fn from(e: io::Error) -> Self {
        CryptoError { inner: IOError(e) }
    }
}
