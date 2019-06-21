use std::{convert::TryFrom, num, result};

use core::Class;
use nom::*;
use serde::{
    de::{self, Deserialize, DeserializeSeed, EnumAccess, SeqAccess, VariantAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::{tag::Tag, value::*, error::{Result, Error}};

pub fn from_slice<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(bytes);

    T::deserialize(&mut deserializer)
}

struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {

    fn from_slice(input: &'de [u8]) -> Self {
        Self { input }
    }

    /// Looks for the next tag but doesn't advance the slice.
    fn look_at_tag(&mut self) -> Result<Tag> {
        Ok(parse_identifier_octet(self.input)?.1)
    }

    fn parse_value(&mut self) -> Result<Value<&'de [u8]>> {
        let (slice, contents) = parse_value(self.input)?;
        self.input = slice;

        Ok(contents)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let value = self.parse_value()?;

        if value.contents.len() != 1 {
            panic!("Incorrect length for boolean")
        }

        // TODO: This logic changes for DER & CER.
        Ok(match value.contents[0] {
            0 => false,
            _ => true,
        })
    }

    fn parse_integer<T: FromStrRadix>(&mut self) -> Result<T> {
        let value = self.parse_value()?;

        let mut radix_str = String::with_capacity(value.contents.len() * 8);

        for byte in value.contents {
            radix_str.push_str(&format!("{:08b}", byte));
        }

        Ok(T::from_str_radix(&radix_str, 2)?)
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.look_at_tag()? {
            Tag::EOC => return Err(Error::Custom("Unexpected End Of contents.".into())),
            Tag::BOOL => self.deserialize_bool(visitor),
            Tag::INTEGER => self.deserialize_i64(visitor),
            Tag::BIT_STRING => self.deserialize_newtype_struct("BitString", visitor),
            Tag::OCTET_STRING => self.deserialize_bytes(visitor),
            Tag::NULL => self.deserialize_unit(visitor),
            Tag::SEQUENCE => self.deserialize_seq(visitor),
            Tag::OBJECT_IDENTIFIER => self.deserialize_newtype_struct("ObjectIdentifier", visitor),
            // Tag::REAL,
            // Tag::ENUMERATED => self.deserialize_,
            Tag::UTF8_STRING => self.deserialize_str(visitor),
            tag => panic!("TAG: {:?}", tag), // _ => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.parse_integer()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.parse_integer()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.parse_integer()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.parse_integer()?)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(self.parse_integer()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.parse_integer()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.parse_integer()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.parse_integer()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.parse_integer()?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.parse_value()?;

        visitor.visit_str(&*String::from_utf8_lossy(value.contents))
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &str, visitor: V) -> Result<V::Value> {
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &str, visitor: V) -> Result<V::Value> {
        match name {
            "ASN.1#OctetString" => {
                visitor.visit_seq(OctetString::new(self.parse_value()?.contents))
            }
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &str, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }


    fn deserialize_struct<V: Visitor<'de>>(self, _name: &str, fields: &[&str], visitor: V) -> Result<V::Value> {
        visitor.visit_seq(Sequence::new(self.parse_value()?.contents, fields.len()))
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.parse_value()?;
        visitor.visit_seq(Sequence::new(value.contents, None))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
        where
             V: Visitor<'de>,
    {
        let tag = self.look_at_tag()?;

        if let Some(variant) = variants.get(tag.tag) {
            visitor.visit_enum(&mut Enum::new(variant, self.input))
        } else {
            panic!("Variant not found")
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
    {
        self.parse_value()?;
        visitor.visit_unit()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
    {
        let value = self.parse_value()?;
        visitor.visit_seq(OctetString::new(value.contents))
    }

    forward_to_deserialize_any! {
        ignored_any identifier
    }
}

struct OctetString<'de> {
    contents: &'de [u8]
}

impl<'de> OctetString<'de> {
    fn new(contents: &'de [u8]) -> Self {
        Self { contents }
    }
}

impl<'de> SeqAccess<'de> for OctetString<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        use serde::de::value::SeqDeserializer;
        seed.deserialize(SeqDeserializer::new(self.contents.into_iter().cloned())).map(Some)
    }
}

struct Sequence<'de> {
    de: Deserializer<'de>,
    elements: Option<usize>,
}

impl<'de> Sequence<'de> {
    fn new<I: Into<Option<usize>>>(input: &'de [u8], elements: I) -> Self {
        let de = Deserializer::from_slice(input);
        let elements = elements.into();

        Self { de, elements }
    }
}

impl<'de> SeqAccess<'de> for Sequence<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(ref mut elements) = self.elements {
            if *elements == 0 {
                return Ok(None)
            }

            *elements -= 1;
        } else if self.de.input.is_empty() {
            return Ok(None)
        }

        seed.deserialize(&mut self.de).map(Some)
    }
}

struct Enum<'de> {
    de: Deserializer<'de>,
    variant: &'static str,
}

impl<'de> Enum<'de> {
    fn new(variant: &'static str, input: &'de [u8]) -> Self {
        let de = Deserializer::from_slice(input);

        Self { variant, de, }
    }
}

impl<'a, 'de> EnumAccess<'de> for &'a mut Enum<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        use serde::de::IntoDeserializer;
        let val: Result<_> = seed.deserialize(self.variant.into_deserializer());

        Ok((val?, self))
    }
}

impl<'a, 'de> VariantAccess<'de> for &'a mut Enum<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut self.de, visitor)
    }
}

trait FromStrRadix: Sized {
    fn from_str_radix(src: &str, radix: u32) -> result::Result<Self, num::ParseIntError>;
}

macro_rules! integers {
    ($($num:ty)*) => {
        $(
        impl FromStrRadix for $num {
            fn from_str_radix(src: &str, radix: u32) -> result::Result<Self, num::ParseIntError> {
                u128::from_str_radix(src, radix).map(|n| n as $num)
            }
        }
        )*
    }
}

integers!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

named!(
    parse_initial_octet<Tag>,
    bits!(do_parse!(
        class: map!(take_bits!(u8, 2), Class::try_from)
            >> is_constructed: map!(take_bits!(u8, 1), is_constructed)
            >> tag: take_bits!(usize, 5)
            >> (Tag::new(class.expect("Invalid class"), is_constructed, tag))
    ))
);

named!(pub(crate) parse_identifier_octet<Tag>, do_parse!(
    identifier: parse_initial_octet >>
    // 31 is 5 bits set to 1.
    long_tag: cond!(identifier.tag >= 31, do_parse!(
        body: take_while!(is_part_of_octet) >>
        end: take!(1) >>
        result: value!(parse_tag(&body, end[0])) >>
        (result)
    )) >>

    (identifier.set_tag(long_tag.unwrap_or(identifier.tag)))
));

named!(
    parse_contents,
    do_parse!(length: take!(1) >> contents: apply!(take_contents, length[0]) >> (&contents))
);

named!(pub(crate) parse_value<&[u8], Value<&[u8]>>, do_parse!(
    tag: parse_identifier_octet >>
    contents: parse_contents >>
    (Value::new(tag, contents))
));

fn is_constructed(byte: u8) -> bool {
    byte != 0
}

fn is_part_of_octet(input: u8) -> bool {
    input & 0x80 != 0
}

fn parse_tag(body: &[u8], end: u8) -> usize {
    let mut tag = 0;

    for byte in body {
        tag <<= 7;
        tag |= (byte & 0x7F) as usize;
    }

    tag <<= 7;
    // end doesn't need to be bitmasked as we know the MSB is `0` (X.690 8.1.2.4.2.a).
    tag |= end as usize;

    tag
}

fn concat_bits(body: &[u8], width: u8) -> usize {
    let mut result: usize = 0;

    for byte in body {
        result <<= width;
        result |= *byte as usize;
    }

    result
}

fn take_contents(input: &[u8], length: u8) -> IResult<&[u8], &[u8]> {
    if length == 128 {
        take_until_and_consume!(input, &[0, 0][..])
    } else if length >= 127 {
        let length = length ^ 0x80;
        do_parse!(
            input,
            length: take!(length)
                >> result: value!(concat_bits(&length, 8))
                >> contents: take!(result)
                >> (contents)
        )
    } else {
        take!(input, length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::types::{ObjectIdentifier, OctetString};
    use serde_derive::Deserialize;

    macro_rules! variant_tests {
        ($($test_fn:ident : {$($fn_name:ident ($input:expr) == $expected:expr);+;})+) => {
            $(
                $(
                    #[test]
                    fn $fn_name() {
                        let (rest, result) = $test_fn($input.into()).unwrap();
                        eprintln!("REST {:?}", rest);
                        assert_eq!($expected, result);
                    }
                )+
            )+
        }
    }

    variant_tests! {
        parse_identifier_octet: {
            universal_bool(&[0x1][..]) == Tag::new(Class::Universal, false, 1);
            private_primitive(&[0xC0][..]) == Tag::new(Class::Private, false, 0);
            context_constructed(&[0xA0][..]) == Tag::new(Class::Context, true, 0);
            private_long_constructed(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F][..])
                == Tag::new(Class::Private, true, 0x1FFFFFFFFFFFF);
        }

        parse_value: {
            primitive_bool(&[0x1, 0x1, 0xFF][..]) == Value::<&[u8]>::new(Tag::new(Class::Universal, false, 1), &[0xff]);
        }
    }

    #[test]
    fn value_long_length_form() {
        let (_, value) = parse_value([0x1, 0x81, 0x2, 0xF0, 0xF0][..].into()).unwrap();

        assert_eq!(value.contents, &[0xF0, 0xF0]);
    }

    #[test]
    fn value_really_long_length_form() {
        let full_buffer = [0xff; 0x100];

        let mut value = vec![0x1, 0x82, 0x1, 0x0];
        value.extend_from_slice(&full_buffer);

        let (_, value) = parse_value((&*value).into()).unwrap();

        assert_eq!(value.contents, &full_buffer[..]);
    }

    #[test]
    fn value_indefinite_length_form() {
        let (_, value) = parse_value([0x1, 0x80, 0xf0, 0xf0, 0xf0, 0xf0, 0, 0][..].into()).unwrap();

        assert_eq!(value.contents, &[0xf0, 0xf0, 0xf0, 0xf0]);
    }

    #[test]
    fn pkcs12_to_value() {
        let _ = parse_value((&*std::fs::read("tests/data/test.p12").unwrap()).into()).unwrap();
    }

    #[test]
    fn bool() {
        let yes: bool = super::from_slice(&[0x1, 0x1, 0xFF][..]).unwrap();
        let no: bool = super::from_slice(&[0x1, 0x1, 0x0][..]).unwrap();

        assert!(yes);
        assert!(!no);
    }

    #[test]
    fn choice() {
        #[derive(Clone, Debug, Deserialize, PartialEq)]
        enum Foo {
            Ein,
            Zwei,
            Drei,
        }

        assert_eq!(Foo::Ein, from_slice(&[0x80, 0][..]).unwrap());
        assert_eq!(Foo::Zwei, from_slice(&[0x81, 0][..]).unwrap());
        assert_eq!(Foo::Drei, from_slice(&[0x82, 0][..]).unwrap());
    }

    #[test]
    fn fixed_array_as_sequence() {
        let array = [8u8; 4];
        let raw = &[48, 4*3, 2, 1, 8, 2, 1, 8, 2, 1, 8, 2, 1, 8][..];
        assert_eq!(array, from_slice::<[u8; 4]>(&raw).unwrap());
    }

    #[test]
    fn choice_newtype_variant() {
        #[derive(Clone, Debug, Deserialize, PartialEq)]
        enum Foo {
            Bar(bool),
            Baz(OctetString),
        }

        let os = OctetString::from(vec![1, 2, 3, 4, 5]);

        assert_eq!(Foo::Bar(true), from_slice(&[0x80, 1, 0xff][..]).unwrap());
        assert_eq!(os, from_slice(&[0x81, 5, 1, 2, 3, 4, 5][..]).unwrap());
    }

    /*
    #[test]
    fn oid_from_bytes() {
        let oid = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();
        let from_raw =
            crate::from_slice(&[0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..]).unwrap();

        assert_eq!(oid, from_raw);
    }
    */
}
