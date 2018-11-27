use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;

use serde::de;
use serde::ser;

pub struct Error(Box<Inner>);

pub(crate) enum Inner {
	/// Catchall for syntax error messages
	Message(Box<str>),

	/// Some IO error occurred while serializing or deserializing.
	Io(io::Error),

	/// EOF while parsing a primitive.
	EofWhileParsingPrimitive,

	/// EOF while parsing a construction.
	EofWhileParsingConstruction,

	/// Encountered nesting of ASN.1 constructions more than 128 layers deep.
	RecursionLimitExceeded,
}

/// Categorizes the cause of a `serde_json::Error`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Category {
	/// The error was caused by a failure to read or write bytes on an IO
	/// stream.
	Io,

	/// The error was caused by input that was not syntactically valid JSON.
	Syntax,

	/// The error was caused by input data that was semantically incorrect.
	///
	/// For example, JSON containing a number is semantically incorrect when the
	/// type being deserialized into holds a String.
	Data,

	/// The error was caused by prematurely reaching the end of the input data.
	///
	/// Callers that process streaming input may be interested in retrying the
	/// deserialization once more data is available.
	Eof,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
	/// Categorizes the cause of this error.
	///
	/// - `Category::Io` - failure to read or write bytes on an IO stream
	/// - `Category::Syntax` - input that is not syntactically valid encoding
	/// - `Category::Data` - input data that is semantically incorrect
	/// - `Category::Eof` - unexpected end of the input data
	pub fn classify(&self) -> Category {
		match *self.0 {
			Inner::Message(_) =>
				Category::Data,

			Inner::Io(_) =>
				Category::Io,

			Inner::EofWhileParsingPrimitive |
			Inner::EofWhileParsingConstruction =>
				Category::Eof,

			Inner::RecursionLimitExceeded =>
				Category::Syntax,
		}
	}

	/// Returns true if this error was caused by a failure to read or write
	/// bytes on an IO stream.
	pub fn is_io(&self) -> bool {
		self.classify() == Category::Io
	}

	/// Returns true if this error was caused by input that was not
	/// syntactically valid JSON.
	pub fn is_syntax(&self) -> bool {
		self.classify() == Category::Syntax
	}

	/// Returns true if this error was caused by input data that was
	/// semantically incorrect.
	///
	/// For example, JSON containing a number is semantically incorrect when the
	/// type being deserialized into holds a String.
	pub fn is_data(&self) -> bool {
		self.classify() == Category::Data
	}

	/// Returns true if this error was caused by prematurely reaching the end of
	/// the input data.
	///
	/// Callers that process streaming input may be interested in retrying the
	/// deserialization once more data is available.
	pub fn is_eof(&self) -> bool {
		self.classify() == Category::Eof
	}
}

impl From<Error> for io::Error {
	fn from(j: Error) -> Self {
		if let Inner::Io(err) = *j.0 {
			return err;
		}

		match j.classify() {
			Category::Io => unreachable!(),
			Category::Syntax | Category::Data => io::Error::new(io::ErrorKind::InvalidData, j),
			Category::Eof => io::Error::new(io::ErrorKind::UnexpectedEof, j),
		}
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error(Box::new(Inner::Io(value)))
	}
}

impl Error {
	#[cold]
	pub(crate) fn syntax(inner: Inner) -> Self {
		Error(Box::new(inner))
	}

	#[cold]
	pub(crate) fn io(error: io::Error) -> Self {
		Error(Box::new(Inner::Io(error)))
	}
}

impl Display for Inner {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Inner::Message(ref msg) => f.write_str(msg),
			Inner::Io(ref err) => Display::fmt(err, f),
			Inner::EofWhileParsingPrimitive => f.write_str("EOF while parsing a primitive"),
			Inner::EofWhileParsingConstruction => f.write_str("EOF while parsing a construction"),
			Inner::RecursionLimitExceeded => f.write_str("recursion limit exceeded"),
		}
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		if let Inner::Io(ref err) = *self.0 {
			error::Error::description(err)
		}
		else {
			"ASN.1 error"
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		if let Inner::Io(ref err) = *self.0 {
			Some(err)
		}
		else {
			None
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		Display::fmt(&*self.0, f)
	}
}

impl Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Error({:?})", self.0.to_string())
	}
}

impl de::Error for Error {
	#[cold]
	fn custom<T: Display>(msg: T) -> Error {
		Error(Box::new(Inner::Message(msg.to_string().into_boxed_str())))
	}

	#[cold]
	fn invalid_type(unexp: de::Unexpected, exp: &de::Expected) -> Self {
		if let de::Unexpected::Unit = unexp {
			Error::custom(format_args!("invalid type: null, expected {}", exp))
		}
		else {
			Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
		}
	}
}

impl ser::Error for Error {
	#[cold]
	fn custom<T: Display>(msg: T) -> Error {
		Error(Box::new(Inner::Message(msg.to_string().into_boxed_str())))
	}
}
