mod decoder;
mod encoder;
mod value;

pub use crate::decoder::Decoder;
pub use value::Value;

pub use encoder::to_der;
pub use decoder::from_der;

use bytes::{BufMut, BytesMut};
use core::Tag;

#[derive(Clone, Debug)]
pub struct Primitive<V: AsRef<[u8]>> {
    pub implicit: Tag,
    pub explicit: Option<Tag>,
    pub value: V,
}

impl<V: AsRef<[u8]>> Primitive<V> {
    pub fn new(implicit: Tag, value: V) -> Primitive<V> {
        Primitive {
            implicit: implicit,
            explicit: None,
            value: value,
        }
    }

    pub fn explicit(mut self, tag: Tag) -> Self {
        self.explicit = Some(tag);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Construct<B: BufMut = BytesMut> {
    pub implicit: Tag,
    pub explicit: Option<Tag>,
    pub buffer: B,
}

impl<B: BufMut> Construct<B> {
    pub fn new(implicit: Tag) -> Construct<BytesMut> {
        Construct {
            implicit: implicit.constructed(true),
            explicit: None,
            buffer: BytesMut::new(),
        }
    }

    pub fn with_buffer(implicit: Tag, buffer: B) -> Construct<B> {
        Construct {
            implicit: implicit,
            explicit: None,
            buffer: buffer,
        }
    }

    pub fn explicit(mut self, tag: Tag) -> Self {
        self.explicit = Some(tag);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use decoder::parse_value;

    #[test]
    fn bool() {
        assert_eq!(true, from_der(&to_der(true)).unwrap());
        assert_eq!(false, from_der(&to_der(false)).unwrap());
    }

    macro_rules! integer_tests {
        ($($name:ident : $integer:ty),*) => {
            $(
                #[test]
                fn $name () {
                    let min = <$integer>::min_value();
                    let max = <$integer>::max_value();

                    assert_eq!(min, from_der(&to_der(min)).unwrap());
                    assert_eq!(max, from_der(&to_der(max)).unwrap());
                }
            )*
        }
    }

    integer_tests! {
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        u128: u128

    }

    #[test]
    fn integer() {
        assert_eq!(0i64, from_der(&to_der(0i64)).unwrap());
        assert_eq!(127i64, from_der(&to_der(127i64)).unwrap());
        assert_eq!(128i64, from_der(&to_der(128i64)).unwrap());
        assert_eq!(-128i64, from_der(&to_der(-128i64)).unwrap());
        assert_eq!(-128i8, from_der(&to_der(-128i8)).unwrap());
        assert_eq!(256i64, from_der(&to_der(256i64)).unwrap());
    }
}
