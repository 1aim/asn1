use std::{
    convert::{TryFrom, TryInto},
    error, fmt,
};

use failure::Fallible;
use nom::*;
use serde::{
    de::{self, Deserialize, DeserializeSeed, SeqAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::tag::Tag;
use crate::value::*;
use core::Class;

pub fn from_der<'a, T>(bytes: &'a [u8]) -> Fallible<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_der(bytes);

    Ok(T::deserialize(&mut deserializer)?)
}

pub fn from_der_partial<'a, T>(bytes: &'a [u8]) -> Fallible<(&'a [u8], T)>
where
    T: TryFrom<Value<&'a [u8]>, Error = failure::Error>,
{
    let (slice, value) = parse_value(bytes).unwrap();

    Ok((slice, value.try_into()?))
}


struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {

    fn from_der(input: &'de [u8]) -> Self {
        Self { input }
    }

    /// Looks for the next tag but doesn't advance the slice.
    fn look_at_tag(&mut self) -> Tag {
        let (_, tag) = parse_identifier_octet(self.input).unwrap();

        tag
    }

    fn parse_tag(&mut self) -> Tag {
        let (slice, tag) = parse_identifier_octet(self.input).unwrap();
        self.input = slice;

        tag
    }

    fn parse_value(&mut self) -> Value<&'de [u8]> {
        let (slice, contents) = parse_value(self.input).unwrap();
        self.input = slice;

        contents
    }

    fn parse_bool(&mut self) -> bool {
        let value = self.parse_value();

        if value.contents.len() != 1 {
            panic!("Incorrect length for boolean")
        }

        // TODO: This logic changes for DER & CER.
        match value.contents[0] {
            0 => false,
            _ => true,
        }
    }

    fn parse_integer<T: FromStrRadix>(&mut self) -> T {
        let value = self.parse_value();

        let mut radix_str = String::with_capacity(value.contents.len() * 8);

        for byte in value.contents {
            radix_str.push_str(&format!("{:08b}", byte));
        }

        T::from_str_radix(&radix_str, 2).unwrap()
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.look_at_tag() {
            Tag::EOC => return Err(Error::Custom("Unexpected end Of contents.".into())),
            Tag::BOOL => self.deserialize_bool(visitor),
            Tag::INTEGER => self.deserialize_i64(visitor),
            Tag::BIT_STRING => self.deserialize_newtype_struct("BitString", visitor),
            Tag::OCTET_STRING => self.deserialize_bytes(visitor),
            Tag::NULL => self.deserialize_unit(visitor),
            Tag::OBJECT_IDENTIFIER => self.deserialize_newtype_struct("ObjectIdentifier", visitor),
            // Tag::REAL,
            // Tag::ENUMERATED => self.deserialize_,
            Tag::UTF8_STRING => self.deserialize_str(visitor),
            _ => unimplemented!(), // visitor.visit_struct(self),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.parse_bool())
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i8(self.parse_integer())
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i16(self.parse_integer())
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.parse_integer())
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i64(self.parse_integer())
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i128(self.parse_integer())
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.parse_integer())
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u16(self.parse_integer())
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.parse_integer())
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.parse_integer())
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u128(self.parse_integer())
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _name: &str, fields: &[&str], visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(Sequence::new(self.parse_value().contents, fields.len()))
    }

    forward_to_deserialize_any! {
        f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

struct Sequence<'de> {
    de: Deserializer<'de>,
    elements: usize,
}

impl<'de> Sequence<'de> {
    fn new(input: &'de [u8], elements: usize) -> Self {
        let de = Deserializer::from_der(input);

        Self { de, elements }
    }
}

impl<'de> SeqAccess<'de> for Sequence<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.elements == 0 {
            return Ok(None)
        }

        self.elements -= 1;

        seed.deserialize(&mut self.de).map(Some)
    }
}

#[derive(Debug)]
enum Error {
    Custom(String),
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "Unknown Error: {}", msg),
        }
    }
}

trait FromStrRadix: Sized {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError>;
}

macro_rules! integers {
    ($($num:ty)*) => {
        $(
        impl FromStrRadix for $num {
            fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
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
    use core::types::ObjectIdentifier;

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
    fn value_to_bool() {
        let yes: bool = super::from_der(&[0x1, 0x1, 0xFF][..]).unwrap();
        let no: bool = super::from_der(&[0x1, 0x1, 0x0][..]).unwrap();

        assert!(yes);
        assert!(!no);
    }

    #[test]
    fn value_long_length() {
        let (_, value) = parse_value([0x1, 0x81, 0x2, 0xF0, 0xF0][..].into()).unwrap();

        assert_eq!(value.contents, &[0xF0, 0xF0]);
    }

    #[test]
    fn value_really_long_length() {
        let full_buffer = [0xff; 0x100];

        let mut value = vec![0x1, 0x82, 0x1, 0x0];
        value.extend_from_slice(&full_buffer);

        let (_, value) = parse_value((&*value).into()).unwrap();

        assert_eq!(value.contents, &full_buffer[..]);
    }

    #[test]
    fn value_indefinite_length() {
        let (_, value) = parse_value([0x1, 0x80, 0xf0, 0xf0, 0xf0, 0xf0, 0, 0][..].into()).unwrap();

        assert_eq!(value.contents, &[0xf0, 0xf0, 0xf0, 0xf0]);
    }

    #[test]
    fn pkcs12_to_value() {
        let _ = parse_value((&*std::fs::read("tests/data/test.p12").unwrap()).into()).unwrap();
    }

    #[test]
    fn oid_from_bytes() {
        let (_, value) =
            parse_value([0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..].into()).unwrap();
        let oid = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();

        assert_eq!(oid, value.try_into().unwrap());
    }
}
