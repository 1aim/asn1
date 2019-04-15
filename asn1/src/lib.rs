#[allow(unused_imports)]
#[macro_use]
extern crate derive;

#[doc(hidden)]
pub use derive::*;
pub use core::*;

pub mod error;
pub use crate::error::*;

#[cfg(feature = "der")]
pub use impl_der as der;

use std::io;

#[inline]
pub fn to_writer<W, E, T>(writer: &mut W, mut encoder: E, value: T) -> Result<()>
	where W: io::Write, E: core::Encoder + core::Encode<T>
{
	encoder.encode(writer, Value::new(value))?;
	Ok(())
}

#[cfg(feature = "der")]
#[inline]
pub fn to_der<W, T>(writer: &mut W, value: T) -> Result<()>
	where W: io::Write, impl_der::Encoder: Encode<T>
{
	to_writer(writer, impl_der::Encoder, value)
}
