use std::io::{self, prelude::*};
use bytes::{buf, BytesMut, Buf, BufMut, IntoBuf};

use tag::Tag;

#[derive(Clone, Debug)]
pub struct Primitive<V: AsRef<[u8]>> {
	pub implicit: Tag,
	pub explicit: Option<Tag>,
	pub value:    V,
}

impl<V: AsRef<[u8]>> Primitive<V> {
	pub fn new(implicit: Tag, value: V) -> Primitive<V> {
		Primitive {
			implicit: implicit,
			explicit: None,
			value:    value,
		}
	}

	pub fn explicit(mut self, tag: Tag) -> Self {
		self.explicit = Some(tag);
		self
	}
}

#[derive(Clone, Debug)]
pub struct Construct<B: BufMut> {
	pub implicit: Tag,
	pub explicit: Option<Tag>,
	pub buffer:   B,
}

impl<B: BufMut> Construct<B> {
	pub fn new(implicit: Tag) -> Construct<BytesMut> {
		Construct {
			implicit: implicit,
			explicit: None,
			buffer:   BytesMut::new(),
		}
	}

	pub fn with_buffer(implicit: Tag, buffer: B) -> Construct<B> {
		Construct {
			implicit: implicit,
			explicit: None,
			buffer:   buffer,
		}
	}

	pub fn explicit(mut self, tag: Tag) -> Self {
		self.explicit = Some(tag);
		self
	}
}

pub trait Encoder: Clone {
	#[inline]
	fn encode_primitive<W, V>(&mut self, writer: &mut W, primitive: Primitive<V>) -> io::Result<()>
		where W: Write + ?Sized, V: AsRef<[u8]>;

	#[inline]
	fn encode_construct<W, B, F>(&mut self, writer: &mut W, mut construct: Construct<B>, func: F) -> io::Result<()>
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

pub trait Encode<T> {
	fn encode<W>(&mut self, writer: &mut W, value: T) -> io::Result<()>
		where W: Write + ?Sized;
}
