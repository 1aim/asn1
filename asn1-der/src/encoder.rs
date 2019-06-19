use std::{collections::VecDeque, io::Write};

use serde::{ser::{self, Impossible}, Serialize};
use core::Class;

use crate::{error::{Error, Result}, tag::Tag};

pub fn to_writer<W, T>(writer: W, value: &T)
    -> Result<()>
    where W: Write,
          T: Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    Ok(())
}

pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut vec = Vec::new();

    to_writer(&mut vec, value)?;

    Ok(vec)
}

struct Value<'a> {
    tag: Tag,
    contents: &'a [u8]
}

impl<'a> Value<'a> {
    fn new(tag: Tag, contents: &'a [u8]) -> Self {
        Self { tag, contents }
    }
}

pub struct Serializer<W: Write> {
    output: W,
    is_octet_string: bool,
}

impl Serializer<Vec<u8>> {
    fn new_sink() -> Self {
        Self { output: Vec::new(), is_octet_string: false }
    }
}

impl<W: Write> Serializer<W> {
    fn new(output: W) -> Self {
        Self { output, is_octet_string: false }
    }

    fn encode_preamble(&mut self, tag: Tag, original_length: usize) -> Result<()> {
        tag.encode(&mut self.output)?;

        if original_length <= 127 {
            self.output.write(&[original_length as u8])?;
        } else {
            let mut length = original_length;
            let mut length_buffer = Vec::new();

            while length != 0 {
                length_buffer.push((length & 0xff) as u8);
                length >>= 8;
            }

            self.output.write(&[length_buffer.len() as u8 | 0x80])?;
            self.output.write(&length_buffer)?;
        }

        Ok(())
    }

    fn encode_bool(&mut self, v: bool) -> Result<()> {
        let v = if v { 0xff } else { 0 };

        self.encode_value(Value::new(Tag::BOOL, &[v]))
    }

    fn encode_integer(&mut self, mut value: u128) -> Result<()> {
        let mut contents = VecDeque::new();

        if value != 0 {
            if value <= u8::max_value() as u128 {
                contents.push_front(value as u8);
            } else {
                while value != 0 {
                    contents.push_front(value as u8);
                    value = value.wrapping_shr(8);
                }
            }
        } else {
            contents.push_front(0);
        }

        self.encode_value(Value::new(Tag::INTEGER,  &*Vec::from(contents)))
    }

    fn encode_value(&mut self, v: Value) -> Result<()> {
        self.encode_preamble(v.tag, v.contents.len())?;
        self.output.write(v.contents)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Sequence<'a, W>;
    type SerializeTuple = Sequence<'a, W>;
    type SerializeTupleStruct = Sequence<'a, W>;
    type SerializeTupleVariant = Sequence<'a, W>;
    type SerializeMap = Sequence<'a, W>;
    type SerializeStruct = Sequence<'a, W>;
    type SerializeStructVariant = Sequence<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.encode_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.encode_integer(v as u128)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u128(u128::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u128(u128::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u128(u128::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_u128(u128::from(v))
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.encode_integer(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        unimplemented!()
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, _v: &str) -> Result<()> {
        unimplemented!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.encode_value(Value::new(Tag::OCTET_STRING, v))
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        let variant_tag = Tag::new(Class::Context, false, variant_index as usize);
        let mut sink = Serializer::new_sink();
        sink.encode_value(Value::new(Tag::NULL, &[]))?;
        self.encode_value(Value::new(variant_tag, &sink.output))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match name {
            "ASN.1#OctetString" => {
                self.is_octet_string = true;
                value.serialize(self)
            }
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let variant_tag = Tag::new(Class::Context, false, variant_index as usize);
        let mut sink = Serializer::new_sink();

        value.serialize(&mut sink)?;

        self.encode_value(Value::new(variant_tag , &sink.output))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Sequence::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.serialize_seq(len)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_seq(Some(len))
    }
}

pub struct Sequence<'a, W: Write> {
    ser: &'a mut Serializer<W>,
    sink: Serializer<Vec<u8>>,
    raw_sink: RawSerializer<Vec<u8>>,
}

impl<'a, W: Write> Sequence<'a, W> {
    fn new(ser: &'a mut Serializer<W>) -> Self {
        Self {
            ser,
            sink: Serializer::new(Vec::new()),
            raw_sink: RawSerializer::new(Vec::new()),
        }
    }
}

impl<'a, W: Write> ser::SerializeSeq for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
        where T: ?Sized + Serialize
    {
        if self.ser.is_octet_string {
            value.serialize(&mut self.raw_sink)
        } else {
            value.serialize(&mut self.sink)
        }
    }

    fn end(self) -> Result<()> {
        if self.ser.is_octet_string {
            self.ser.is_octet_string = false;
            self.ser.encode_value(Value::new(Tag::OCTET_STRING, &self.raw_sink.output))
        } else {
            self.ser.encode_value(Value::new(Tag::SEQUENCE, &self.sink.output))
        }
    }
}

impl<'a, W: Write> ser::SerializeMap for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
        where T: ?Sized + Serialize
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where T: ?Sized + Serialize
    {
        Ok(())
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: Write> ser::SerializeStruct for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: Write> ser::SerializeTuple for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

// Same thing but for tuple structs.
impl<'a, W: Write> ser::SerializeTupleStruct for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: Write> ser::SerializeTupleVariant for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

/// Serializer used solely to encode octet strings properly.
struct RawSerializer<W> {
    output: W,
}

impl<W: Write> RawSerializer<W> {
    fn new(output: W) -> Self {
        Self { output }
    }
}

impl<'a, W: Write> ser::Serializer for &'a mut RawSerializer<W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_i16(self, _: i16) -> Result<()> {
        unreachable!()
    }

    fn serialize_i32(self, _: i32) -> Result<()> {
        unreachable!()
    }

    fn serialize_i64(self, _: i64) -> Result<()> {
        unreachable!()
    }

    fn serialize_i128(self, _: i128) -> Result<()> {
        unreachable!()
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.write(&[v])?;
        Ok(())
    }

    fn serialize_u16(self, _: u16) -> Result<()> {
        unreachable!()
    }

    fn serialize_u32(self, _: u32) -> Result<()> {
        unreachable!()
    }

    fn serialize_u64(self, _: u64) -> Result<()> {
        unreachable!()
    }

    fn serialize_u128(self, _: u128) -> Result<()> {
        unreachable!()
    }

    fn serialize_f32(self, _: f32) -> Result<()> {
        unreachable!()
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        unreachable!()
    }

    fn serialize_char(self, _: char) -> Result<()> {
        unreachable!()
    }

    fn serialize_str(self, _: &str) -> Result<()> {
        unreachable!()
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        unreachable!()
    }

    fn serialize_none(self) -> Result<()> {
        unreachable!()
    }

    fn serialize_some<T>(self, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_unit(self) -> Result<()> {
        unreachable!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unreachable!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        unreachable!()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unreachable!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unreachable!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unreachable!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        unreachable!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unreachable!()
    }
}


#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};
    use core::types::ObjectIdentifier;

    use super::*;

    #[test]
    fn bool() {
        assert_eq!(&[1, 1, 255][..], &*to_vec(&true).unwrap());
        assert_eq!(&[1, 1, 0][..], &*to_vec(&false).unwrap());
    }

    #[test]
    fn choice() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Foo {
            Ein,
            Zwei,
            Drei,
        }

        assert_eq!(&[0x80, 2, 5, 0][..], &*to_vec(&Foo::Ein).unwrap());

    }

    /*
    #[test]
    fn object_identifier_to_bytes() {
        let itu: Vec<u8> = to_vec(ObjectIdentifier::new(vec![2, 999, 3]).unwrap());
        let rsa: Vec<u8> = to_vec(ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap());

        assert_eq!(&[0x6, 0x3, 0x88, 0x37, 0x03][..], &*itu);
        assert_eq!(&[0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..], &*rsa);
    }
    */
}
