mod raw;

use std::{collections::VecDeque, io::Write};

use serde::{ser, Serialize};
use log::debug;

use crate::{error::{Error, Result}, tag::{Class, Tag}};

use self::raw::RawSerializer;

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

    debug!("HEX Debug representation: {:?}", hex::encode(&vec));

    Ok(vec)
}

pub struct Serializer<W: Write> {
    output: W,
    tag: Option<Tag>,
    implicit: bool,
    constructed: bool,
}

impl Serializer<Vec<u8>> {
    fn serialize_to_vec<T: ?Sized + Serialize>(value: &T, implicit: bool) -> Result<Self> {
        let mut ser = Self::new(Vec::new());
        ser.implicit = implicit;
        value.serialize(&mut ser)?;
        Ok(ser)
    }
}

impl<W: Write> Serializer<W> {
    fn new(output: W) -> Self {
        Self { output, tag: None, implicit: false, constructed: false }
    }

    fn set_tag(&mut self, tag: Tag) {
        self.tag = Some(tag);
    }

    fn set_constructed(&mut self) {
        self.constructed = true;
    }

    fn encode(&mut self, contents: &[u8]) -> Result<()> {
        // TODO: Switch bool to operate on the true version of this expression.
        if !self.implicit {
            self.encode_preamble(contents.len())?;
        }

        self.output.write(contents)?;
        Ok(())
    }

    fn encode_preamble(&mut self, original_length: usize) -> Result<()> {

        let tag = self.tag.take().ok_or(Error::Custom(String::from("no tag present.")))?;
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

        self.set_tag(Tag::BOOL);
        self.encode(&[v])
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

        self.set_tag(Tag::INTEGER);
        self.encode(&Vec::from(contents))
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

    fn serialize_str(self, v: &str) -> Result<()> {
        self.set_tag(Tag::UNIVERSAL_STRING);
        self.encode(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.set_tag(Tag::OCTET_STRING);
        self.encode(v)
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
        self.set_tag(Tag::from_context(self.constructed, variant_index));
        self.encode(&[])
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
                self.set_tag(Tag::OCTET_STRING);
            }
            _ => {}
        }

        value.serialize(self)
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
        let ser = Serializer::serialize_to_vec(value, true)?;
        let is_constructed = ser.tag.map(|t| t.is_constructed).unwrap_or(false);
        let variant_tag = Tag::new(Class::Context, is_constructed, variant_index as usize);
        self.set_tag(variant_tag);
        self.encode(&ser.output)
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
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.set_tag(Tag::from_context(self.constructed, variant_index));
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.set_constructed();
        self.serialize_seq(len)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.set_constructed();
        self.serialize_tuple_variant(name, variant_index, variant, len)
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
        match self.ser.tag {
            Some(Tag::OCTET_STRING) => value.serialize(&mut self.raw_sink),
            _ => value.serialize(&mut self.sink)
        }

    }

    fn end(self) -> Result<()> {
        self.ser.tag = self.ser.tag.or(Some(Tag::SEQUENCE));
        self.ser.constructed = false;

        let contents = match self.ser.tag {
            Some(Tag::OCTET_STRING) => self.raw_sink.output,
            _ => self.sink.output,
        };

        self.ser.encode(&contents)
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


#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};
    use core::types::{ObjectIdentifier, OctetString};

    use super::*;

    #[test]
    fn bool() {
        assert_eq!(&[1, 1, 255][..], &*to_vec(&true).unwrap());
        assert_eq!(&[1, 1, 0][..], &*to_vec(&false).unwrap());
    }

    #[test]
    fn universal_string() {
        assert_eq!(&[28, 5, 0x4A, 0x6F, 0x6E, 0x65, 0x73][..], &*to_vec(&"Jones").unwrap());
    }

    #[test]
    fn fixed_array_as_sequence() {
        let array = [8u8; 4];
        assert_eq!(&[48, 4*3, 2, 1, 8, 2, 1, 8, 2, 1, 8, 2, 1, 8][..], &*to_vec(&array).unwrap());
    }

    #[test]
    fn choice() {
        #[derive(Clone, Debug, Serialize, PartialEq)]
        enum Foo {
            Ein,
            Zwei,
            Drei,
        }

        assert_eq!(&[0x80, 0][..], &*to_vec(&Foo::Ein).unwrap());
        assert_eq!(&[0x81, 0][..], &*to_vec(&Foo::Zwei).unwrap());
        assert_eq!(&[0x82, 0][..], &*to_vec(&Foo::Drei).unwrap());
    }

    #[test]
    fn choice_newtype_variant() {
        #[derive(Clone, Debug, Serialize, PartialEq)]
        enum Foo {
            Bar(bool),
            Baz(OctetString),
            Blah(Blah),
        }

        #[derive(Clone, Debug, Serialize, PartialEq)]
        struct Blah {
            data: OctetString,
        }

        let os = OctetString::from(vec![1, 2, 3, 4, 5]);

        assert_eq!(&[0x80, 1, 0xff][..], &*to_vec(&Foo::Bar(true)).unwrap());
        assert_eq!(&[0x81, 5, 1, 2, 3, 4, 5][..], &*to_vec(&Foo::Baz(os.clone())).unwrap());
        assert_eq!(&[0xA2, 7, 4, 5, 1, 2, 3, 4, 5][..], &*to_vec(&Foo::Blah(Blah { data: os })).unwrap());
    }

    #[test]
    fn sequence_in_sequence_in_choice() {
        use hex::encode;
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Foo {
            Bar {
                data: OctetString,
            }
        }

        let bar = Foo::Bar { data: OctetString::from(vec![1, 2, 3, 4])};

        let raw = &[
            0xA0,
            6,
            0x4,
            4,
            1,
            2,
            3,
            4,
        ][..];

        let result = to_vec(&bar).unwrap();

        assert_eq!(raw, &*result);
        assert_eq!(raw, &*result);
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
