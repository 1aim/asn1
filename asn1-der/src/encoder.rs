mod object_identifier;
mod bit_string;
mod bytes;
mod prefix;

use std::io::Write;

use log::debug;
use num_bigint::ToBigInt;
use serde::{ser, Serialize};

use crate::error::{Error, Result};
use core::identifier::Identifier;

use self::{
    bit_string::BitStringSerializer,
    object_identifier::ObjectIdentifierSerializer,
    bytes::ByteSerializer,
    prefix::PrefixSerializer
};

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    Ok(())
}

/// Serialize an instance of `T` as a ASN.1 DER byte vector.
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut vec = Vec::new();

    to_writer(&mut vec, value)?;

    debug!("HEX Debug representation: {:?}", hex::encode(&vec));

    Ok(vec)
}

pub struct Serializer<W: Write> {
    output: W,
    tag: Option<Identifier>,
    implicit: bool,
    constructed: bool,
    /// If present bool matches `implicit` state.
    prefixed: Option<bool>,
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
        Self {
            output,
            tag: None,
            implicit: false,
            constructed: false,
            prefixed: None,
        }
    }

    fn set_tag(&mut self, tag: Identifier) {
        self.tag = Some(tag);
    }

    fn set_constructed(&mut self) {
        self.constructed = true;
    }

    fn clear_state(&mut self) {
        self.tag = None;
        self.constructed = false;
        self.implicit = false;
        self.prefixed = None;
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
        let tag = self
            .tag
            .take()
            .ok_or(Error::Custom(String::from("no tag present.")))?;

        self.encode_tag(tag)?;

        if original_length <= 127 {
            self.output.write(&[original_length as u8])?;
        } else {
            let mut length = original_length;
            let mut length_buffer = std::collections::VecDeque::new();

            while length != 0 {
                length_buffer.push_front((length & 0xff) as u8);
                length >>= 8;
            }

            let length_buffer: Vec<u8> = length_buffer.into();
            self.output.write(&[length_buffer.len() as u8 | 0x80])?;
            self.output.write(&length_buffer)?;
        }

        self.clear_state();

        Ok(())
    }

    fn encode_tag(&mut self, tag: Identifier) -> Result<()> {
        let mut tag_byte = tag.class as u8;
        let mut tag_number = tag.tag;

        // Constructed is a single bit.
        tag_byte <<= 1;
        tag_byte |= match tag {
            Identifier::EXTERNAL |
            Identifier::SEQUENCE |
            Identifier::SET => 1,
            _ if self.constructed => 1,
            _ => 0,
        };

        // Identifier number is five bits
        tag_byte <<= 5;

        if tag_number >= 0x1f {
            tag_byte |= 0x1f;
            self.output.write(&[tag_byte])?;

            while tag_number != 0 {
                let mut encoded_number: u8 = (tag_number & 0x7f) as u8;
                tag_number >>= 7;

                // Fill the last bit unless we're at the last bit.
                if tag_number != 0 {
                    encoded_number |= 0x80;
                }

                self.output.write(&[encoded_number])?;
            }
        } else {
            tag_byte |= tag_number as u8;
            self.output.write(&[tag_byte])?;
        }

        Ok(())
    }

    fn encode_bool(&mut self, v: bool) -> Result<()> {
        let v = if v { 0xff } else { 0 };

        self.set_tag(Identifier::BOOL);
        self.encode(&[v])
    }

    fn encode_integer<N: ToBigInt>(&mut self, value: N) -> Result<()> {
        self.set_tag(Identifier::INTEGER);
        self.encode(&value.to_bigint().unwrap().to_signed_bytes_be())
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
        log::trace!("Serializing bool.");
        self.encode_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        log::trace!("Serializing i8.");
        self.encode_integer(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        log::trace!("Serializing i16.");
        self.encode_integer(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        log::trace!("Serializing i32.");
        self.encode_integer(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        log::trace!("Serializing i64.");
        self.encode_integer(v)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        log::trace!("Serializing i128.");
        self.encode_integer(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        log::trace!("Serializing u8.");
        self.encode_integer(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        log::trace!("Serializing u16.");
        self.encode_integer(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        log::trace!("Serializing u32.");
        self.encode_integer(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        log::trace!("Serializing u64.");
        self.encode_integer(v)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        log::trace!("Serializing u128.");
        self.encode_integer(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        log::trace!("Serializing f32.");
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        log::trace!("Serializing f64.");
        unimplemented!()
    }

    fn serialize_char(self, v: char) -> Result<()> {
        log::trace!("Serializing char.");
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        log::trace!("Serializing str.");
        self.set_tag(Identifier::UNIVERSAL_STRING);
        self.encode(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        log::trace!("Serializing bytes.");
        self.set_tag(Identifier::OCTET_STRING);
        self.encode(v)
    }

    fn serialize_none(self) -> Result<()> {
        log::trace!("Serializing none.");
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        log::trace!("Serializing some.");
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        log::trace!("Serializing unit.");
        self.set_tag(Identifier::NULL);
        self.encode(&[])
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        log::trace!("Serializing unit struct.");
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        log::trace!("Serializing unit variant.");
        if self.tag.map(|i| i == Identifier::ENUMERATED).unwrap_or(false) {
            self.encode(&variant_index.to_bigint().unwrap().to_signed_bytes_be())
        } else {
           unreachable!("Shouldn't be possible.")
        }

    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match name {
            "ASN.1#OctetString" => {
                log::trace!("Serializing OCTET STRING.");
                self.set_tag(Identifier::OCTET_STRING);
            }
            "ASN.1#ObjectIdentifier" => {
                log::trace!("Serializing OBJECT IDENTIFIER.");
                self.set_tag(Identifier::OBJECT_IDENTIFIER);
            }
            "ASN.1#BitString" => {
                log::trace!("Serializing BIT STRING.");
                self.set_tag(Identifier::BIT_STRING);
            }
            "ASN.1#Integer" => {
                log::trace!("Serializing INTEGER.");
                self.set_tag(Identifier::INTEGER);
            }
            "ASN.1#Enumerated" => {
                log::trace!("Serializing ENUMERATED.");
                self.set_tag(Identifier::ENUMERATED);
            }
            "ASN.1#Implicit" => {
                log::trace!("Serializing implicit prefix.");
                self.prefixed = Some(true);
            }
            "ASN.1#Explicit" => {
                log::trace!("Serializing explicit prefix.");
                self.set_constructed();
                self.prefixed = Some(false);
            }
            name => {
                log::trace!("Serializing {}.", name);
            }
        }

        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        log::trace!("Serializing {}.", name);
        let ser = Serializer::serialize_to_vec(value, true)?;
        self.constructed = ser.constructed;
        self.set_tag(ser.tag.unwrap());
        self.encode(&ser.output)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        log::trace!("Serializing seq");
        Ok(Sequence::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        log::trace!("Serializing tuple");
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        log::trace!("Serializing tuple struct");
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        log::trace!("Serializing {}", name);
        self.set_tag(Identifier::from_context(variant_index));
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        log::trace!("Serializing map");
        self.set_constructed();
        self.serialize_seq(len)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        log::trace!("Serializing {}", name);
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        log::trace!("Serializing {}", name);
        self.set_constructed();
        self.serialize_tuple_variant(name, variant_index, variant, len)
    }
}

pub struct Sequence<'a, W: Write> {
    ser: &'a mut Serializer<W>,
    sink: SerializerKind,
}

impl<'a, W: Write> Sequence<'a, W> {
    fn new(ser: &'a mut Serializer<W>) -> Self {
        let sink = match ser.tag {
            Some(Identifier::OCTET_STRING) => {
                SerializerKind::OctetString(ByteSerializer::new())
            }
            Some(Identifier::OBJECT_IDENTIFIER) => {
                SerializerKind::ObjectIdentifier(ObjectIdentifierSerializer::new())
            }
            Some(Identifier::BIT_STRING) => {
                SerializerKind::BitString(BitStringSerializer::new())
            }
            Some(Identifier::INTEGER) => {
                SerializerKind::Integer(ByteSerializer::new())
            }
            _ => {
                match ser.prefixed {
                    Some(implicit) => SerializerKind::Prefix(PrefixSerializer::new(implicit)),
                    _ => SerializerKind::Normal(Serializer::new(Vec::new())),
                }
            },
        };

        Self { ser, sink }
    }
}

impl<'a, W: Write> ser::SerializeSeq for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.sink.serialize(value)
    }

    fn end(self) -> Result<()> {
        self.ser.tag = match self.sink {
            SerializerKind::Prefix(ref ser) => {
                if ser.output.constructed {
                    self.ser.set_constructed();
                }

                Some(Identifier::new(ser.class.unwrap(), ser.tag.unwrap()))
            }
            _ => self.ser.tag.or(Some(Identifier::SEQUENCE)),
        };

        let contents = self.sink.output();
        self.ser.encode(&contents)
    }
}

impl<'a, W: Write> ser::SerializeMap for Sequence<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
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

enum SerializerKind {
    BitString(BitStringSerializer),
    Integer(ByteSerializer),
    Normal(Serializer<Vec<u8>>),
    ObjectIdentifier(ObjectIdentifierSerializer),
    OctetString(ByteSerializer),
    Prefix(PrefixSerializer),
}

impl SerializerKind {
    fn serialize<T: ser::Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        match self {
            SerializerKind::BitString(ser) => value.serialize(ser),
            SerializerKind::Integer(ser) => value.serialize(ser),
            SerializerKind::Normal(ser) => value.serialize(ser),
            SerializerKind::ObjectIdentifier(ser) => value.serialize(ser),
            SerializerKind::OctetString(ser) => value.serialize(ser),
            SerializerKind::Prefix(ser) => value.serialize(ser),
        }
    }

    fn output(self) -> Vec<u8> {
        match self {
            SerializerKind::BitString(mut ser) => {
                ser.output.insert(0, ser.last.trailing_zeros() as u8);
                ser.output
            },
            SerializerKind::Normal(ser) => ser.output,
            SerializerKind::Integer(ser) => ser.output,
            SerializerKind::OctetString(ser) => ser.output,
            SerializerKind::ObjectIdentifier(ser) => ser.output,
            SerializerKind::Prefix(ser) => ser.output.output,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{identifier::constant::*, types::*};
    use serde_derive::{Deserialize, Serialize};
    use typenum::consts::*;

    use super::*;

    #[test]
    fn bool() {
        assert_eq!(&[1, 1, 255][..], &*to_vec(&true).unwrap());
        assert_eq!(&[1, 1, 0][..], &*to_vec(&false).unwrap());
    }

    #[test]
    fn integer() {
        let small_integer = Integer::from(5);
        let multi_byte_integer = Integer::from(0xffff);
        assert_eq!(&[2, 1, 5][..], &*to_vec(&small_integer).unwrap());
        assert_eq!(&[2, 3, 0, 0xff, 0xff][..], &*to_vec(&multi_byte_integer).unwrap());
    }

    #[test]
    fn universal_string() {
        assert_eq!(
            &[28, 5, 0x4A, 0x6F, 0x6E, 0x65, 0x73][..],
            &*to_vec(&"Jones").unwrap()
        );
    }

    #[test]
    fn fixed_array_as_sequence() {
        let array = [8u8; 4];
        assert_eq!(
            &[48, 4 * 3, 2, 1, 8, 2, 1, 8, 2, 1, 8, 2, 1, 8][..],
            &*to_vec(&array).unwrap()
        );
    }

    #[test]
    fn encode_long_sequence() {
        let vec = vec![5; 0xffff];
        let preamble = vec![0x30u8, 0x83, 0x2, 0xFF, 0xFD];
        assert_eq!(&*preamble, &to_vec(&vec).unwrap()[..preamble.len()]);
    }

    #[test]
    fn enumerated() {
        use core::types::{Enumerable, Enumerated};
        #[derive(Clone, Debug, Serialize, PartialEq)]
        enum Foo {
            Ein,
            Zwei,
            Drei,
        }

        impl Enumerable for Foo {}

        let ein = Enumerated::new(Foo::Ein);
        let zwei = Enumerated::new(Foo::Zwei);
        let drei = Enumerated::new(Foo::Drei);

        assert_eq!(&[0xA, 1, 0][..], &*to_vec(&ein).unwrap());
        assert_eq!(&[0xA, 1, 1][..], &*to_vec(&zwei).unwrap());
        assert_eq!(&[0xA, 1, 2][..], &*to_vec(&drei).unwrap());
    }

    #[test]
    fn choice() {
        #[derive(Clone, Debug, Serialize, PartialEq)]
        enum Foo {
            Ein(Implicit<Context, U0, ()>),
            Zwei(Implicit<Context, U1, ()>),
            Drei((Implicit<Context, U2, ()>)),
        }

        assert_eq!(&[0x80, 0][..], &*to_vec(&Foo::Ein(Implicit::new(()))).unwrap());
        assert_eq!(&[0x81, 0][..], &*to_vec(&Foo::Zwei(Implicit::new(()))).unwrap());
        assert_eq!(&[0x82, 0][..], &*to_vec(&Foo::Drei(Implicit::new(()))).unwrap());
    }

    #[test]
    fn choice_newtype_variant() {
        #[derive(Clone, Debug, Serialize, PartialEq)]
        enum Foo {
            Bar(Implicit<Context, U0, bool>),
            Baz(Implicit<Context, U1, OctetString>),
            Blah(Implicit<Context, U2, Blah>),
        }

        #[derive(Clone, Debug, Serialize, PartialEq)]
        struct Blah {
            data: OctetString,
        }

        let os = OctetString::from(vec![1, 2, 3, 4, 5]);

        assert_eq!(&[0x80, 1, 0xff][..], &*to_vec(&Foo::Bar(Implicit::new(true))).unwrap());
        assert_eq!(
            &[0x81, 5, 1, 2, 3, 4, 5][..],
            &*to_vec(&Foo::Baz(Implicit::new(os.clone()))).unwrap()
        );
        assert_eq!(
            &[0xA2, 7, 4, 5, 1, 2, 3, 4, 5][..],
            &*to_vec(&Foo::Blah(Implicit::new(Blah { data: os }))).unwrap()
        );
    }

    #[test]
    fn sequence_in_sequence_in_choice() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Foo {
            Bar { data: OctetString },
        }

        let bar = Foo::Bar {
            data: OctetString::from(vec![1, 2, 3, 4]),
        };

        let raw = &[0xA0, 6, 0x4, 4, 1, 2, 3, 4][..];

        let result = to_vec(&bar).unwrap();

        assert_eq!(raw, &*result);
        assert_eq!(raw, &*result);
    }

    #[test]
    fn object_identifier() {
        use core::types::ObjectIdentifier;

        let just_root: Vec<u8> = to_vec(&ObjectIdentifier::new(vec![1, 2]).unwrap()).unwrap();
        let itu: Vec<u8> = to_vec(&ObjectIdentifier::new(vec![2, 999, 3]).unwrap()).unwrap();
        let rsa: Vec<u8> =
            to_vec(&ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap()).unwrap();

        assert_eq!(&[0x6, 0x1, 0x2a][..], &*just_root);
        assert_eq!(&[0x6, 0x3, 0x88, 0x37, 0x03][..], &*itu);
        assert_eq!(&[0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..], &*rsa);
    }

    #[test]
    fn sequence_with_option() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            a: u8,
            b: Option<u8>,
        }

        let some = Foo { a: 1, b: Some(2) };
        let none = Foo { a: 1, b: None };

        assert_eq!(
            &[0x30, 3 * 2, 0x2, 0x1, 0x1, 0x2, 0x1, 0x2][..],
            &*to_vec(&some).unwrap()
        );
        assert_eq!(
            &[0x30, 0x5, 0x2, 0x1, 0x1, 0x5, 0x0][..],
            &*to_vec(&none).unwrap()
        );
    }

    #[test]
    fn bit_string() {
        use core::types::BitString;

        let bitvec = BitString::from_bytes(&[0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0]);

        assert_eq!(
            &[0x3u8, 0x7, 0x04, 0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0][..],
            &*to_vec(&bitvec).unwrap()
        );
    }

    #[test]
    fn implicit_prefix() {
        use typenum::consts::*;
        use core::identifier::constant::*;
        type MyInteger = core::types::Implicit<Universal, U7, u64>;

        let new_int = MyInteger::new(5);

        assert_eq!(&[7, 1, 5], &*to_vec(&new_int).unwrap());
    }

    #[test]
    fn explicit_prefix() {
        use typenum::consts::*;
        use core::identifier::constant::*;
        type MyInteger = core::types::Explicit<Context, U0, u64>;

        let new_int = MyInteger::new(5);

        assert_eq!(&[0xA0, 3, 2, 1, 5], &*to_vec(&new_int).unwrap());
    }
}
