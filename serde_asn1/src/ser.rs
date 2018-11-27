use std::io::prelude::*;
use std::fmt;

use serde::ser::{self, Impossible, Serialize};
use bytes::{buf, BytesMut, BufMut};

use error::{Error, Result};
use asn1::{self, encoder::{Encoder as Super, Encode, Primitive}, tag::{self, Tag}};

pub struct Serializer<W, E> {
	writer:  W,
  encoder: E,
}

pub trait Encoder = asn1::Encoder +
	asn1::Encode<()> + asn1::Encode<bool> +
	asn1::Encode<i8> + asn1::Encode<i16> + asn1::Encode<i32> + asn1::Encode<i64> + asn1::Encode<i128> +
	asn1::Encode<u8> + asn1::Encode<u16> + asn1::Encode<u32> + asn1::Encode<u64> + asn1::Encode<u128> +
	asn1::Encode<f32> + asn1::Encode<f64> +
	for<'a> asn1::Encode<&'a str> + for<'a> asn1::Encode<&'a [u8]>;

impl<W: Write, E: Encoder> Serializer<W, E> {
	pub fn new(writer: W, encoder: E) -> Self {
		Serializer { writer, encoder }
	}
}

impl<'a, W, E> ser::Serializer for &'a mut Serializer<W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Construct<'a, W, E>;
	type SerializeTuple = Construct<'a, W, E>;
	type SerializeTupleStruct = Construct<'a, W, E>;
	type SerializeTupleVariant = Construct<'a, W, E>;
	type SerializeMap = Construct<'a, W, E>;
	type SerializeStruct = Construct<'a, W, E>;
	type SerializeStructVariant = Construct<'a, W, E>;

	#[inline]
	fn serialize_bool(self, value: bool) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i8(self, value: i8) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i16(self, value: i16) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i32(self, value: i32) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_i64(self, value: i64) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	fn serialize_i128(self, value: i128) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u8(self, value: u8) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u16(self, value: u16) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u32(self, value: u32) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_u64(self, value: u64) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	fn serialize_u128(self, value: u128) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_f32(self, value: f32) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_f64(self, value: f64) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
		Ok(())
	}

	#[inline]
	fn serialize_char(self, value: char) -> Result<()> {
		unimplemented!()
	}

	#[inline]
	fn serialize_str(self, value: &str) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
    Ok(())
	}

	#[inline]
	fn serialize_bytes(self, value: &[u8]) -> Result<()> {
		self.encoder.encode(&mut self.writer, value)?;
    Ok(())
	}

	#[inline]
	fn serialize_unit(self) -> Result<()> {
		self.encoder.encode(&mut self.writer, ())?;
		Ok(())
	}

	#[inline]
	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		unimplemented!()
	}

	#[inline]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<()> {
		unimplemented!()
	}

	/// Serialize newtypes without an object wrapper.
	#[inline]
	fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
		where T: Serialize
	{
		unimplemented!()
	}

	#[inline]
	fn serialize_newtype_variant<T: ?Sized>(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<()>
		where T: Serialize
	{
		unimplemented!()
	}

	#[inline]
	fn serialize_none(self) -> Result<()> {
		unimplemented!()
	}

	#[inline]
	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
		where T: Serialize
	{
		unimplemented!()
	}

	#[inline]
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		Ok(Construct::new(self))
	}

	#[inline]
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		Ok(Construct::new(self))
	}

	#[inline]
	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		Ok(Construct::new(self))
	}

	#[inline]
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		Ok(Construct::with_explicit(self, Tag::context(variant_index as u8)))
	}

	#[inline]
	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		Ok(Construct::new(self))
	}

	#[inline]
	fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
		Ok(Construct::new(self))
	}

	#[inline]
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant> {
		Ok(Construct::with_explicit(self, Tag::context(variant_index as u8)))
	}

	fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
		where T: fmt::Display
	{
		unimplemented!()
	}
}

pub struct Construct<'a, W: 'a, E: 'a> {
	explicit: Option<Tag>,
	parent:   &'a mut Serializer<W, E>,
	current:  Serializer<buf::Writer<BytesMut>, E>,
}

impl<'a, W: 'a, E: 'a> Construct<'a, W, E>
	where W: Write, E: Encoder
{
	pub fn new(parent: &mut Serializer<W, E>) -> Construct<W, E> {
		Construct {
			explicit: None,
			current: Serializer {
				writer:  BytesMut::new().writer(),
				encoder: parent.encoder.clone(),
			},

			parent: parent,
		}
	}

	pub fn with_explicit(parent: &mut Serializer<W, E>, explicit: Tag) -> Construct<W, E> {
		Construct {
			explicit: Some(explicit),
			current: Serializer {
				writer:  BytesMut::new().writer(),
				encoder: parent.encoder.clone(),
			},

			parent: parent,
		}
	}
}

impl<'a, W, E> ser::SerializeSeq for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
		where T: Serialize
	{
		value.serialize(&mut self.current)
	}

	#[inline]
	fn end(mut self) -> Result<()> {
		self.current.encoder.encode_primitive(&mut self.parent.writer, Primitive {
			implicit: tag::SEQUENCE,
			explicit: self.explicit,
			value:    self.current.writer.get_ref()
		})?;

		Ok(())
	}
}

impl<'a, W, E> ser::SerializeTuple for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: Serialize,
	{
		ser::SerializeSeq::serialize_element(self, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		ser::SerializeSeq::end(self)
	}
}

impl<'a, W, E> ser::SerializeTupleStruct for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: Serialize,
	{
		ser::SerializeSeq::serialize_element(self, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		ser::SerializeSeq::end(self)
	}
}

impl<'a, W, E> ser::SerializeTupleVariant for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: Serialize,
	{
		ser::SerializeSeq::serialize_element(self, value)
	}

	#[inline]
	fn end(self) -> Result<()> {
		unimplemented!();
	}
}

impl<'a, W, E> ser::SerializeMap for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
		where T: Serialize,
	{
		unimplemented!();
	}

	#[inline]
	fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
		where T: Serialize,
	{
		unimplemented!();
	}

	#[inline]
	fn end(self) -> Result<()> {
		unimplemented!();
	}
}

impl<'a, W, E> ser::SerializeStruct for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
		where T: Serialize
	{
		value.serialize(&mut self.current)
	}

	#[inline]
	fn end(mut self) -> Result<()> {
		self.current.encoder.encode_primitive(&mut self.parent.writer, Primitive {
			implicit: tag::SEQUENCE,
			explicit: self.explicit,
			value:    self.current.writer.get_ref()
		})?;

		Ok(())
	}
}

impl<'a, W, E> ser::SerializeStructVariant for Construct<'a, W, E>
	where W: Write, E: Encoder
{
	type Ok = ();
	type Error = Error;

	#[inline]
	fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
		where T: Serialize,
	{
		value.serialize(&mut self.current)
	}

	#[inline]
	fn end(mut self) -> Result<()> {
		self.current.encoder.encode_primitive(&mut self.parent.writer, Primitive {
			implicit: tag::SEQUENCE,
			explicit: self.explicit,
			value:    self.current.writer.get_ref()
		})?;

		Ok(())
	}
}
