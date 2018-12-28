#![feature(trait_alias)]

extern crate serde;
extern crate bytes;
extern crate asn1;

pub mod error;
pub use crate::error::*;

pub mod ser;
pub use crate::ser::Serializer;

use std::io::prelude::*;
use serde::Serialize;

#[inline]
pub fn to_writer<W, E, T>(writer: W, encoder: E, value: &T) -> Result<()>
	where W: Write, E: ser::Encoder, T: ?Sized + Serialize
{
	let mut ser = ser::Serializer::new(writer, encoder);
	value.serialize(&mut ser)?;
	Ok(())
}

#[inline]
pub fn to_der<W, T>(writer: W, value: &T) -> Result<()>
	where W: Write, T: ?Sized + Serialize
{
	to_writer(writer, asn1::der::Encoder, value)
}
