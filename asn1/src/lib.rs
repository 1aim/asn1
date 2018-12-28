#[cfg(feature = "asn1_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate asn1_derive;
#[cfg(feature = "asn1_derive")]
#[doc(hidden)]
pub use asn1_derive::*;

pub mod error;
pub use crate::error::*;

pub mod tag;
mod support;
pub use crate::support::*;

pub mod encoder;
pub use crate::encoder::{Encoder, Encode};

pub mod der;

use std::io::prelude::*;

#[inline]
pub fn to_writer<W, E, T>(writer: &mut W, mut encoder: E, value: T) -> Result<()>
	where W: Write, E: Encoder + Encode<T>
{
	encoder.encode(writer, value)?;
	Ok(())
}

#[inline]
pub fn to_der<W, T>(writer: &mut W, value: T) -> Result<()>
	where W: Write, der::Encoder: Encode<T>
{
	to_writer(writer, der::Encoder, value)
}
