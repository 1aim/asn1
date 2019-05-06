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
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn bool() {
        assert_eq!(true, from_der(&to_der(true)).unwrap().try_into().unwrap());
        assert_eq!(false, from_der(&to_der(false)).unwrap().try_into().unwrap());
    }
}
