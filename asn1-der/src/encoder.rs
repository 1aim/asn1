use std::io::{self, prelude::*};
use bigint::{BigInt, BigUint};
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use bytes::{buf, Buf, BufMut, IntoBuf};

use core::{Encoder as Super, Encode, Value};
use core::tag::{self, Tag};
use core::ObjectId;
use crate::{Construct, Primitive};

#[derive(Copy, Clone, Debug, Default)]
pub struct Encoder;

impl Super for Encoder {
	const CANONICAL: bool = true;
}

impl Encoder {
	pub fn encode_primitive<W, V>(&mut self, writer: &mut W, primitive: Primitive<V>) -> io::Result<()>
		where W: Write + ?Sized, V: AsRef<[u8]>
	{
		encode_header(writer, primitive.implicit, primitive.explicit, primitive.value.as_ref().len())?;
		writer.write_all(primitive.value.as_ref())?;

		Ok(())
	}

	pub fn encode_construct<W, B, F>(&mut self, writer: &mut W, mut construct: Construct<B>, func: F) -> io::Result<()>
		where W: Write + ?Sized, B: IntoBuf + BufMut, F: for<'a> FnOnce(buf::Writer<&'a mut B>, &mut Self) -> io::Result<()>
	{
		func((&mut construct.buffer).writer(), self)?;

		self.encode_primitive(writer, Primitive {
			implicit: construct.implicit,
			explicit: construct.explicit,
			value:    construct.buffer.into_buf().bytes()
		})
	}
}

fn encode_length<W: Write + ?Sized>(writer: &mut W, length: usize) -> io::Result<()> {
	if length >= 128 {
		let n = {
			let mut i = length;
			let mut bytes = 1;

			while i > 255 {
				bytes += 1;
				i >>= 8;
			}

			bytes
		};

		writer.write_all(&[0x80 | n])?;

		for i in (1 .. n + 1).rev() {
			writer.write_all(&[(length >> ((i - 1) * 8)) as u8])?;
		}
	}
	else {
		writer.write_all(&[length as u8])?;
	}

	Ok(())
}

fn encode_tag<W: Write + ?Sized>(writer: &mut W, tag: Tag) -> io::Result<()> {
	use core::tag::Class::*;

	let class = match tag.class {
		Universal   => 0 << 6,
		Application => 1 << 6,
		Context     => 2 << 6,
		Private     => 3 << 6,
	};

	let constructed = if tag.constructed {
		0x20
	}
	else {
		0
	};

	writer.write_all(&[class | constructed | tag.number])
}

fn encode_header<W: Write + ?Sized>(writer: &mut W, implicit: Tag, explicit: Option<Tag>, length: usize) -> io::Result<()> {
	if let Some(tag) = explicit {
		encode_tag(writer, tag)?;
	}

	encode_tag(writer, implicit)?;
	encode_length(writer, length)?;

	Ok(())
}

fn encode_base128<W: Write + ?Sized>(writer: &mut W, value: &BigUint) -> io::Result<()> {
	let ZERO: BigUint = BigUint::zero();

	if value == &ZERO {
		return writer.write_all(&[0]);
	}

	let mut length = 0;
	let mut acc    = value.clone();

	while acc > ZERO {
		length += 1;
		acc     = acc >> 7;
	}

	writer.write_all(&[(value & BigUint::from_u8(0x7f).unwrap()).to_u8().unwrap()])?;

	for i in (1 .. length).rev() {
		writer.write_all(&[((value >> (i * 7))
			& BigUint::from_u8(0x7f).unwrap()
			| BigUint::from_u8(0x80).unwrap()).to_u8().unwrap()])?;
	}

	Ok(())
}

impl Encode<()> for Encoder {
	fn encode<W>(&mut self, writer: &mut W, value: Value<()>) -> io::Result<()>
		where W: Write + ?Sized
	{
		self.encode_primitive(writer, Primitive {
			implicit: value.implicit.unwrap_or(tag::NULL),
			explicit: value.explicit,
			value:    b"",
		})
	}
}

impl Encode<bool> for Encoder {
	fn encode<W>(&mut self, writer: &mut W, value: Value<bool>) -> io::Result<()>
		where W: Write + ?Sized
	{
		self.encode_primitive(writer, Primitive {
			implicit: value.implicit.unwrap_or(tag::BOOLEAN),
			explicit: value.explicit,
			value: if *value { &[0xff] } else { &[0x00] }
		})
	}
}

macro_rules! integer {
	() => ();

	($ty:ty) => (
		impl Encode<$ty> for Encoder {
			fn encode<W>(&mut self, writer: &mut W, value: Value<$ty>) -> io::Result<()>
				where W: Write + ?Sized
			{
				let int = BigInt::from(*value);
				self.encode(writer, value.map(|_| &int))
			}
		}
	);

	($ty:ty, $($rest:tt)+) => (
		integer!($ty);
		integer!($($rest)*);
	);
}

integer!(i8, i16, i32, i64, i128);
integer!(u8, u16, u32, u64, u128);

impl<'a> Encode<&'a BigInt> for Encoder {
	fn encode<W>(&mut self, writer: &mut W, value: Value<&'a BigInt>) -> io::Result<()>
		where W: Write + ?Sized
	{
		println!("{}", *value);

		self.encode_primitive(writer, Primitive {
			implicit: value.implicit.unwrap_or(tag::INTEGER),
			explicit: value.explicit,
			value:    &value.to_signed_bytes_be()
		})
	}
}

macro_rules! float {
	() => ();

	($ty:ty) => (
		impl Encode<$ty> for Encoder {
			fn encode<W>(&mut self, _writer: &mut W, _value: Value<$ty>) -> io::Result<()>
				where W: Write + ?Sized
			{
				unimplemented!("floats not supported yet");
			}
		}
	);

	($ty:ty, $($rest:tt)+) => (
		float!($ty);
		float!($($rest)*);
	);
}

float!(f32, f64);

impl<'a> Encode<&'a str> for Encoder {
	fn encode<W>(&mut self, writer: &mut W, value: Value<&'a str>) -> io::Result<()>
		where W: Write + ?Sized
	{
		// TODO(meh): need UTF8 whatever string here
		self.encode_primitive(writer, Primitive {
			implicit: value.implicit.unwrap_or(tag::OCTET_STRING),
			explicit: value.explicit,
			value:    value.into_inner(),
		})
	}
}

impl<'a> Encode<&'a [u8]> for Encoder {
	fn encode<W>(&mut self, writer: &mut W, value: Value<&'a [u8]>) -> io::Result<()>
		where W: Write + ?Sized
	{
		self.encode_primitive(writer, Primitive {
			implicit: value.implicit.unwrap_or(tag::OCTET_STRING),
			explicit: value.explicit,
			value:    value.into_inner(),
		})
	}
}

impl<'a> Encode<&'a ObjectId> for Encoder {
	fn encode<W>(&mut self, writer: &mut W, value: Value<&'a ObjectId>) -> io::Result<()>
		where W: Write + ?Sized
	{
		let first  = (*value).as_ref()[0].to_u8().expect("ObjectId invariants not respected");
		let second = (*value).as_ref()[1].to_u8().expect("ObjectId invariants not respected");

		let mut body = vec![(first * 40) + second];
		for part in (*value).as_ref().iter().skip(2) {
			encode_base128(&mut body, part)?;
		}

		self.encode_primitive(writer, Primitive::new(tag::OBJECT_ID, &body))
	}
}