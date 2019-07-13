//! When serialising or deserialising ASN.1 goes wrong.
use std::{error, fmt, io};

use nom::Err;
use serde::{de, ser};

/// Alias for a `Result` with the error type `asn1_der::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// This type represents all possible errors that can occur when serialising or
/// deserialising ASN.1.
#[derive(Debug)]
pub enum Error {
    /// An unknown error from `serde`.
    Custom(String),
    /// Incorrect length of content bytes for a provided ASN.1 type.
    IncorrectLength(String),
    /// Failure to read or write bytes to an IO stream.
    Io(io::Error),
    /// No enum variant found matching the tag when deserialising.
    NoVariantFound(usize),
    /// Couldn't cast big integer down to primitive numeric.
    IntegerOverflow(String),
    /// Malformed ASN.1 DER.
    Parser(String),
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "Unknown Error: {}", msg),
            Error::IncorrectLength(kind) => write!(f, "Incorrect length for {}", kind),
            Error::Io(error) => write!(f, "IO: {}", error),
            Error::NoVariantFound(index) => write!(f, "No variant found with index '{}'.", index),
            Error::Parser(msg) => write!(f, "Parsing: {}", msg),
            Error::IntegerOverflow(number) => write!(f, "Couldn't cast big int to {}", number),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl<I: std::fmt::Debug> From<Err<I>> for Error {
    fn from(nom_error: Err<I>) -> Self {
        Error::Parser(format!("{:?}", nom_error))
    }
}
