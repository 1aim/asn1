use std::collections::VecDeque;
use std::convert::TryFrom;

use failure::{ensure, Fallible};

use crate::tag::Tag;
use core::types::ObjectIdentifier;

type OwnedValue = Value<Vec<u8>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Value<A: AsRef<[u8]>> {
    pub tag: Tag,
    pub contents: A,
}

impl<A: AsRef<[u8]>> Value<A> {
    pub fn new(tag: Tag, contents: A) -> Self {
        Self { tag, contents }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.tag.is_constructed || self.tag.tag != 1 {
            return None;
        }

        Some(match self.contents.as_ref()[0] {
            0 => false,
            _ => true,
        })
    }

    pub fn len(&self) -> usize {
        self.tag.len() + self.contents.as_ref().len()
    }
}

impl From<bool> for OwnedValue {
    fn from(from: bool) -> Self {
        let byte = match from {
            false => 0u8,
            true => 0xff,
        };
        Value::new(Tag::BOOL, vec![byte])
    }
}

impl<A: AsRef<[u8]>> TryFrom<Value<A>> for bool {
    type Error = failure::Error;

    fn try_from(value: Value<A>) -> Fallible<Self> {
        let contents = value.contents.as_ref();

        ensure!(
            value.tag == Tag::BOOL,
            "{:?} is not tagged as a boolean",
            value.tag
        );

        ensure!(
            contents.len() == 1,
            "Incorrect length for boolean {:?}",
            contents.len()
        );

        Ok(match contents[0] {
            0xff => true,
            _ => false,
        })
    }
}

impl<A: AsRef<[u128]>> From<ObjectIdentifier<A>> for OwnedValue {
    fn from(oid: ObjectIdentifier<A>) -> Self {
        let oid = oid.as_ref();
        let mut buffer = Vec::with_capacity(oid.len());
        let mut iter = oid.into_iter();

        macro_rules! encode_component {
            ($number:expr) => {{
                let mut number = $number;
                let mut bytes = Vec::new();

                while number != 0 {
                    bytes.push(number & 0x7f);
                    number >>= 7;
                }

                for byte in bytes.iter().skip(1).rev() {
                    let octet = (0x80 | byte) as u8;
                    buffer.push(octet);
                    number >>= 7;
                }

                let final_octet = bytes[0] as u8;
                buffer.push(final_octet);
            }};
        }

        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        encode_component!(first * 40 + second);

        for &byte in iter {
            encode_component!(byte);
        }

        Value::new(Tag::OBJECT_IDENTIFIER, buffer)
    }
}

impl<A: AsRef<[u8]>> TryFrom<Value<A>> for ObjectIdentifier<Vec<u128>> {
    type Error = failure::Error;

    fn try_from(value: Value<A>) -> Fallible<Self> {
        ensure!(
            value.tag == Tag::OBJECT_IDENTIFIER,
            "{:?} is not tagged as a object identifier.",
            value.tag
        );
        let contents = value.contents.as_ref();
        ensure!(contents.len() >= 1, "ObjectIdentifier length less than 1.");

        let mut iter = contents.into_iter().map(|&x| x);
        let first_octet = iter.next().unwrap();
        let second_component = first_octet % 40;
        let first_component = ((first_octet - second_component) / 40) as u128;
        let mut oid = vec![first_component, second_component as u128];
        let mut component: u128 = 0;

        for byte in iter {
            component <<= 7;
            component |= (byte & 0x7F) as u128;

            if byte & 0x80 == 0 {
                oid.push(component);
                component = 0;
            }
        }

        Ok(ObjectIdentifier::new(oid)?)
    }
}

impl<A: AsRef<[u8]>> TryFrom<Value<A>> for Vec<u8> {
    type Error = failure::Error;

    fn try_from(value: Value<A>) -> Fallible<Self> {
        ensure!(
            value.tag == Tag::OCTET_STRING,
            "{:?} is not tagged as a octet string.",
            value.tag
        );

        Ok(value.contents.as_ref().to_owned())
    }
}

impl From<Vec<u8>> for OwnedValue {
    fn from(vec: Vec<u8>) -> Self {
        Value::new(Tag::OCTET_STRING, vec)
    }
}

macro_rules! impl_integer {
    ($($integer:ty $( : $unsigned:ty)?),*) => {
        $(
            #[allow(unused_mut)]
            impl From<$integer> for OwnedValue {
                fn from(mut value: $integer) -> Self {
                    let mut contents = VecDeque::new();

                    $(
                        let mut value = value as $unsigned;
                    )?

                    if value != 0 {
                        if std::mem::size_of::<$integer>() == 1 {
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

                    Value::new(Tag::INTEGER,  contents.into())
                }
            }

            impl<A: AsRef<[u8]>> TryFrom<Value<A>> for $integer {
                type Error = failure::Error;
                fn try_from(value: Value<A>) -> Result<Self, Self::Error> {
                    ensure!(value.tag == Tag::INTEGER, "{:?} is not tagged as a INTEGER", value.tag);
                    let contents = value.contents.as_ref();

                    let mut bit_string = String::new();

                    for byte in contents {
                        bit_string.push_str(&format!("{:08b}", byte));
                    }

                    let integer = u128::from_str_radix(&bit_string, 2)?;

                    Ok(integer as $integer)
                }
            }
        )*
    }
}

impl_integer! {
    u8,
    u16,
    u32,
    u64,
    u128,
    i8: u8,
    i16: u16,
    i32: u32,
    i64: u64,
    i128: u128
}
