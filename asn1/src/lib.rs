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
pub fn to_writer<W, E, T, V>(writer: &mut W, mut encoder: E, value: V) -> Result<()>
	where W: io::Write, E: Encoder + Encode<T>, V: Into<Value<T>>
{
	encoder.encode(writer, value.into())?;
	Ok(())
}

#[cfg(feature = "der")]
#[inline]
pub fn to_der<W, T, V>(writer: &mut W, value: V) -> Result<()>
	where W: io::Write, der::Encoder: Encode<T>, V: Into<Value<T>>
{
	to_writer(writer, der::Encoder, value)
}
