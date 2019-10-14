mod bit_string;
mod object_identifier;
mod octet_string;
mod option;
mod prefix;
pub(crate) mod parser;

use std::{fmt, num, result};

use core::identifier::Identifier;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde::{
    de::{self, Deserialize, DeserializeSeed, EnumAccess, SeqAccess, VariantAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::{
    error::{Error, Result},
    identifier::BerIdentifier,
};
use self::{
    bit_string::BitString,
    object_identifier::ObjectIdentifier,
    option::IdentifierDeserializer,
    octet_string::OctetString,
    prefix::Prefix,
};

/// Deserialize an instance of `T` from bytes of ASN.1 DER.
pub fn from_slice<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    log::trace!("Starting deserialisation: {:?}", bytes);
    let mut deserializer = Deserializer::from_slice(bytes);

    T::deserialize(&mut deserializer)
}

/// An untyped ASN.1 value.
#[derive(Debug, PartialEq)]
pub(crate) struct Value<'a> {
    pub identifier: BerIdentifier,
    pub contents: &'a [u8],
}

impl<'a> Value<'a> {
    fn new<I: Into<BerIdentifier>>(identifier: I, contents: &'a [u8]) -> Self {
        Self { identifier: identifier.into(), contents }
    }
}

pub(crate) struct Deserializer<'de> {
    input: &'de [u8],
    enumerated: bool,
    type_check: bool,
}

impl<'de> Deserializer<'de> {
    fn from_slice(input: &'de [u8]) -> Self {
        log::trace!("New Deserializer with input: {:?}", input);
        Self { input, enumerated: false, type_check: true, }
    }

    /// Looks for the next tag but doesn't advance the slice.
    fn peek_at_identifier(&self) -> Result<BerIdentifier> {
        let identifier = parser::parse_identifier_octet(self.input)?.1;
        log::trace!("Peeking at: {:?}", identifier);
        Ok(identifier)
    }

    fn _peek_value(&self) -> Result<Value<'de>> {
        Ok(parser::parse_value(self.input)?.1)
    }

    fn parse_value(&mut self, expected: Option<Identifier>) -> Result<Value<'de>> {
        log::trace!("Attempting to parse: {:?}", self.input);
        let (slice, value) = parser::parse_value(self.input)?;
        self.input = slice;

        if self.type_check {
            if let Some(expected) = expected {
                let actual = *value.identifier;
                if expected != actual {
                    return Err(Error::IncorrectType { expected, actual })
                }
            }
        } else {
            self.type_check = true;
        }

        log::trace!("Value: {:?}", value);
        log::trace!("Remaining: {:?}", self.input);

        Ok(value)
    }

    fn parse_decodable(&mut self, slice: &[u8])

    fn parse_bool(&mut self) -> Result<bool> {
        let value = self.parse_value(Some(Identifier::BOOL))?;

        if value.contents.len() == 1 {
            // TODO: This logic changes for DER & CER.
            Ok(match value.contents[0] {
                0 => false,
                _ => true,
            })
        } else {
            Err(Error::IncorrectLength(String::from("bool")))
        }
    }

    fn parse_integer(&mut self, check: bool) -> Result<BigInt> {
        let expected = if check {
            Some(Identifier::INTEGER)
        } else {
            None
        };

        let value = self.parse_value(expected)?;

        Ok(BigInt::from_signed_bytes_be(&value.contents))
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.peek_at_identifier()?.identifier {
            Identifier::EOC => return Err(Error::Custom("Unexpected End Of contents.".into())),
            Identifier::BOOL => self.deserialize_bool(visitor),
            Identifier::INTEGER => self.deserialize_i64(visitor),
            Identifier::BIT_STRING => self.deserialize_newtype_struct("ASN.1#BitString", visitor),
            Identifier::OCTET_STRING => self.deserialize_bytes(visitor),
            Identifier::NULL => self.deserialize_unit(visitor),
            Identifier::SEQUENCE => self.deserialize_seq(visitor),
            Identifier::OBJECT_IDENTIFIER => {
                self.deserialize_newtype_struct("ASN.1#ObjectIdentifier", visitor)
            }
            // Identifier::REAL,
            // Identifier::ENUMERATED => self.deserialize_,
            Identifier::UTF8_STRING => self.deserialize_str(visitor),
            _ => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising bool.");
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i8.");
        let value = self.parse_integer(true)?
                       .to_i8()
                       .ok_or_else(|| Error::IntegerOverflow("i8".into()))?;

        visitor.visit_i8(value)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i16.");
        let value = self.parse_integer(true)?
                        .to_i16()
                        .ok_or_else(|| Error::IntegerOverflow("i16".into()))?;

        visitor.visit_i16(value)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i32.");
        let value = self.parse_integer(true)?
                        .to_i32()
                        .ok_or_else(|| Error::IntegerOverflow("i32".into()))?;

        visitor.visit_i32(value)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i64.");
        let value = self.parse_integer(true)?
                        .to_i64()
                        .ok_or_else(|| Error::IntegerOverflow("i64".into()))?;

        visitor.visit_i64(value)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising i128.");
        let value = self.parse_integer(true)?
                        .to_i128()
                        .ok_or_else(|| Error::IntegerOverflow("i128".into()))?;

        visitor.visit_i128(value)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u8.");
        let value = self.parse_integer(true)?
                        .to_u8()
                        .ok_or_else(|| Error::IntegerOverflow("u8".into()))?;

        visitor.visit_u8(value)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u16.");
        let value = self.parse_integer(true)?
                        .to_u16()
                        .ok_or_else(|| Error::IntegerOverflow("u16".into()))?;

        visitor.visit_u16(value)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u32.");
        let value = self.parse_integer(true)?
                        .to_u32()
                        .ok_or_else(|| Error::IntegerOverflow("u32".into()))?;

        visitor.visit_u32(value)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u64.");
        let value = self.parse_integer(true)?
                        .to_u64()
                        .ok_or_else(|| Error::IntegerOverflow("u64".into()))?;

        visitor.visit_u64(value)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising u128.");
        let value = self.parse_integer(true)?
                        .to_u128()
                        .ok_or_else(|| Error::IntegerOverflow("u128".into()))?;

        visitor.visit_u128(value)
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
        let value = self.parse_value(Some(Identifier::UNIVERSAL_STRING))?;

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

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising option.");

        let ident = self.peek_at_identifier().map(|i| i.identifier).ok();
        let mut identifier_de = IdentifierDeserializer::new(ident, self);
        let result = visitor.visit_some(&mut identifier_de);

        result
        /*
        let is_none = if self
            .peek_at_identifier()
            .map(|i| i.identifier == Identifier::NULL)
            .unwrap_or(false)
        {
            self.parse_value(Some(Identifier::NULL))?;
            true
        } else {
            self.input.is_empty()
        };

        if is_none {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
        */
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, name: &str, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising unit struct {:?}.", name);
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        match name {
            "ASN.1#OctetString" => {
                log::trace!("Deserialising OCTET STRING.");
                let value = self.parse_value(Some(Identifier::OCTET_STRING))?;
                visitor.visit_seq(OctetString::new(value.contents))
            }
            "ASN.1#ObjectIdentifier" => {
                log::trace!("Deserialising OBJECT IDENTIFIER.");
                let value = self.parse_value(Some(Identifier::OBJECT_IDENTIFIER))?;
                visitor.visit_seq(ObjectIdentifier::new(value.contents))
            }
            "ASN.1#BitString" => {
                log::trace!("Deserialising BIT STRING.");
                let value = self.parse_value(Some(Identifier::BIT_STRING))?;
                visitor.visit_seq(BitString::new(value.contents))
            }
            "ASN.1#Implicit" => {
                log::trace!("Using implicit deserialisation.");
                visitor.visit_seq(Prefix::new(self, false)?)
            }
            "ASN.1#Explicit" => {
                log::trace!("Using explicit deserialisation.");
                visitor.visit_seq(Prefix::new(self, true)?)
            }
            name => {
                log::trace!("Deserialising newtype struct {:?}.", name);

                if name == "ASN.1#Enumerated" {
                    log::trace!("Enabled ENUMERATED decoding");
                    self.enumerated = true;
                }
                visitor.visit_newtype_struct(self)
            }
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising {} length tuple.", len);
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        log::trace!("Deserialising {} length tuple {:?}.", len, name);
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising map.");
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &str,
        fields: &[&str],
        visitor: V,
    ) -> Result<V::Value> {
        log::trace!("Deserialising struct {:?} with fields {:?}.", name, fields);
        let value = self.parse_value(Some(Identifier::SEQUENCE))?;
        visitor.visit_seq(Sequence::new(value.contents, fields.len()))
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        log::trace!("Deserialising sequence.");
        let value = self.parse_value(Some(Identifier::SEQUENCE))?;
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
        log::trace!(
            "Deserialising enum {:?} with variants {:?}.",
            name,
            variants
        );

        let variant_index = if self.enumerated {
            self.enumerated = false;
            self.parse_integer(false)?.to_u32().unwrap()
        } else {
            let identifier = self.peek_at_identifier()?;
            identifier.tag
        };

        let variant = variants.get(variant_index as usize)
                              .ok_or(Error::NoVariantFound(variant_index))?;

        log::trace!("Attempting to deserialise to {}::{}", name, variant);
        visitor.visit_enum(Enum::new(variant, self))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        log::trace!("Deserialising unit");
        self.parse_value(Some(Identifier::NULL))?;
        visitor.visit_unit()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        log::trace!("Deserialising bytes");
        let value = self.parse_value(Some(Identifier::OCTET_STRING))?;
        visitor.visit_seq(OctetString::new(value.contents))
    }

    forward_to_deserialize_any! {
        ignored_any identifier
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
                return Ok(None);
            }

            *elements -= 1;
        } else if self.de.input.is_empty() {
            return Ok(None);
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
        Self { variant, de }
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
        self.de.type_check = false;
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de.type_check = false;
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

struct Inspector<T> {
    marker: std::marker::PhantomData<T>,
}

impl<'de, T> Visitor<'de> for Inspector<T> {
    type Value = BerIdentifier;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a integer")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut visitor: S) -> result::Result<Self::Value, S::Error> {
        let class: u8 = visitor.next_element()?.unwrap();
        let is_constructed: bool = visitor.next_element()?.unwrap();
        let tag: u32 = visitor.next_element()?.unwrap();

        Ok(BerIdentifier::new(core::identifier::Class::from_u8(class), is_constructed, tag))
    }
}

#[cfg(test)]
mod tests {
    use super::from_slice;
    use core::types::*;
    use core::identifier::constant::*;
    use typenum::consts::*;
    use serde_derive::Deserialize;

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
        let raw = &[48, 4 * 3, 2, 1, 8, 2, 1, 8, 2, 1, 8, 2, 1, 8][..];
        assert_eq!(array, from_slice::<[u8; 4]>(&raw).unwrap());
    }

    #[test]
    fn choice_newtype_variant() {
        #[derive(Clone, Debug, Deserialize, PartialEq)]
        enum Foo {
            Bar(Implicit<Context, U0, bool>),
            Baz(Implicit<Context, U1, OctetString>),
        }

        let os = OctetString::from(vec![1, 2, 3, 4, 5]);

        assert_eq!(Foo::Bar(Implicit::new(true)), from_slice(&[0x80, 1, 0xff][..]).unwrap());
        assert_eq!(Foo::Baz(Implicit::new(os)), from_slice(&[0x81, 5, 1, 2, 3, 4, 5][..]).unwrap());
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
