use std::convert::TryFrom;

use failure::ensure;
use core::Class;

type OwnedValue = Value<Vec<u8>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tag {
    pub class: Class,
    pub is_constructed: bool,
    pub tag: usize,
}

impl Tag {
    pub const EOC: Tag = Tag::new(Class::Universal, false, 0);
    pub const BOOL: Tag = Tag::new(Class::Universal, false, 1);
    pub const INTEGER: Tag = Tag::new(Class::Universal, false, 0x02);
    pub const BIT_STRING: Tag = Tag::new(Class::Universal, false, 0x03);
    pub const OCTET_STRING: Tag = Tag::new(Class::Universal, false, 0x04);
    pub const NULL: Tag = Tag::new(Class::Universal, false, 0x05);
    pub const OBJECT_IDENTIFIER: Tag = Tag::new(Class::Universal, false, 0x06);
    pub const SEQUENCE: Tag = Tag::new(Class::Universal, true, 0x10);
    pub const UTC_TIME: Tag = Tag::new(Class::Universal, false, 0x17);
    pub const GENERALIZED_TIME: Tag = Tag::new(Class::Universal, false, 0x18);

    pub const fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self { class, is_constructed, tag }
    }

    pub fn set_tag(mut self, tag: usize) -> Self {
        self.tag = tag;
        self
    }

    pub fn len(&self) -> usize {
        if self.tag > 0x1f {
            2
        } else {
            1
        }
    }
}

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
            return None
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
        let byte = match from { false => 0u8, true => 0xff };
        Value::new(Tag::BOOL, vec![byte])
    }
}

impl<A: AsRef<[u8]>> TryFrom<Value<A>> for bool {
    type Error = failure::Error;

    fn try_from(value: Value<A>) -> core::Result<Self> {
        let contents = value.contents.as_ref();

        ensure!(value.tag == Tag::BOOL, "{:?} is not tagged as a boolean", value.tag);
        ensure!(contents.len() == 1, "Incorrect length for boolean {:?}", contents.len());

        Ok(match contents[0] {
            0xff => true,
            _ => false,
        })

    }
}

