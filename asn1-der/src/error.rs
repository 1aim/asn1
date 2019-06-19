use std::{error, fmt, io};

use nom::Err;
use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Nom(nom::ErrorKind),
    Number(std::num::ParseIntError),
    Custom(String),
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
        use std::error::Error as _;

        match self {
            Error::Custom(msg) => write!(f, "Unknown Error: {}", msg),
            Error::Io(error) => write!(f, "IO: {}", error),
            Error::Nom(error) => write!(f, "Parsing: {}", error.description()),
            Error::Number(error) => write!(f, "Number: {}", error.description()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::Number(error)
    }
}

impl<I> From<Err<I>> for Error {
    fn from(nom_error: Err<I>) -> Self {
        Error::Nom(nom_error.into_error_kind())
    }
}
