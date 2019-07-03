use std::{error, fmt, io};

use nom::Err;
use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    IncorrectLength(String),
    Io(io::Error),
    NoVariantFound(usize),
    Number(std::num::ParseIntError),
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
        use std::error::Error as _;

        match self {
            Error::Custom(msg) => write!(f, "Unknown Error: {}", msg),
            Error::IncorrectLength(kind) => write!(f, "Incorrect length for {}", kind),
            Error::Io(error) => write!(f, "IO: {}", error),
            Error::NoVariantFound(index) => write!(f, "No variant found with index '{}'.", index),
            Error::Parser(msg) => write!(f, "Parsing: {}", msg),
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

impl<I: std::fmt::Debug> From<Err<I>> for Error {
    fn from(nom_error: Err<I>) -> Self {
        Error::Parser(format!("{:?}", nom_error))
    }
}
