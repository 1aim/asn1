pub(crate) mod parser;
use std::{num, result};

use serde::{
    de::{self, Deserialize, DeserializeSeed, EnumAccess, SeqAccess, VariantAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::{tag::Tag, value::*, error::{Result, Error}};
use self::parser::*;

pub fn from_slice<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    log::trace!("Starting deserialisation: {:?}", bytes);
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
        log::trace!("Attempting to parse: {:?}", self.input);
        let (slice, contents) = parse_value(self.input)?;
        self.input = slice;

        log::trace!("Value: {:?}", contents);
        log::trace!("Remaining: {:?}", slice);

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
        log::trace!("Deserialising bool.");
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i8.");
        visitor.visit_i8(self.parse_integer()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i16.");
        visitor.visit_i16(self.parse_integer()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i32.");
        visitor.visit_i32(self.parse_integer()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i64.");
        visitor.visit_i64(self.parse_integer()?)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i128.");
        visitor.visit_i128(self.parse_integer()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u8.");
        visitor.visit_u8(self.parse_integer()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u16.");
        visitor.visit_u16(self.parse_integer()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u32.");
        visitor.visit_u32(self.parse_integer()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u64.");
        visitor.visit_u64(self.parse_integer()?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u128.");
        visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising f32.");
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising f64.");
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising char.");
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising str.");
        let value = self.parse_value()?;

        visitor.visit_str(&*String::from_utf8_lossy(value.contents))
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising string.");
        self.deserialize_str(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising byte buf.");
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising option.");
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &str, _visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising unit struct.");
        unimplemented!() // visitor.visit_u128(self.parse_integer()?)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &str, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising newtype struct {:?}.", name);

        match name {
            "ASN.1#OctetString" => {
                let value = self.parse_value()?;
                visitor.visit_seq(OctetString::new(value.contents))
            }
            _ =>  visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising {} length tuple.", len);
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, name: &str, len: usize, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising {} length tuple {:?}.", len, name);
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising map.");
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(self, name: &str, fields: &[&str], visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising struct {:?} with fields {:?}.", name, fields);
        let value = self.parse_value()?;
        visitor.visit_seq(Sequence::new(value.contents, fields.len()))
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising sequence.");
        let value = self.parse_value()?;
        visitor.visit_seq(Sequence::new(value.contents, None))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
        where
             V: Visitor<'de>,
    {
        log::trace!("Deserialising enum {:?} with variants {:?}.", name, variants);
        let tag = self.look_at_tag()?;

        if let Some(variant) = variants.get(tag.tag) {
            log::trace!("Attempting to deserialise to {}::{}", name, variant);
            visitor.visit_enum(Enum::new(variant, self))
        } else {
            panic!("Variant not found")
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
    {
        log::trace!("Deserialising unit");
        self.parse_value()?;
        visitor.visit_unit()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
    {
        log::trace!("Deserialising bytes");
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

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    variant: &'static str,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(variant: &'static str, de: &'a mut Deserializer<'de>) -> Self {
        Self { variant, de, }
    }
}

impl<'a, 'de> EnumAccess<'de> for Enum<'a, 'de> {
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

impl<'a, 'de> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.de.parse_value()?;
        log::trace!("Deserialised as unit variant.");
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::types::{ObjectIdentifier, OctetString};
    use serde_derive::Deserialize;
    use crate::tag::Class;

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
