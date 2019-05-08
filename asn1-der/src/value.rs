use std::convert::TryFrom;

use failure::{ensure, Fallible};

use crate::tag::Tag;

type OwnedValue = Value<Vec<u8>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Value<A: AsRef<[u8]>> {
    pub(crate) tag: Tag,
    pub(crate) contents: A,
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

macro_rules! impl_integer {
    ($($integer:ty $( : $unsigned:ty)?),*) => {
        $(
            #[allow(unused_mut)]
            impl From<$integer> for Value<Vec<u8>> {
                fn from(mut value: $integer) -> Self {
                    use std::collections::VecDeque;
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