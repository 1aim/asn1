use std::io::Write;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Class {
    Universal = 0,
    Application,
    Context,
    Private,
}

impl From<u8> for Class {
    fn from(value: u8) -> Self {
        match value {
            0 => Class::Universal,
            1 => Class::Application,
            2 => Class::Context,
            3 => Class::Private,
            _ => panic!("Impossible Class"),
        }
    }
}

use crate::error::Result;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tag {
    pub class: Class,
    pub is_constructed: bool,
    pub tag: usize,
}

impl Tag {
    pub const EOC: Tag = Tag::new(Class::Universal, false, 0);
    pub const BOOL: Tag = Tag::new(Class::Universal, false, 1);
    pub const INTEGER: Tag = Tag::new(Class::Universal, false, 2);
    pub const BIT_STRING: Tag = Tag::new(Class::Universal, false, 3);
    pub const OCTET_STRING: Tag = Tag::new(Class::Universal, false, 4);
    pub const NULL: Tag = Tag::new(Class::Universal, false, 5);
    pub const OBJECT_IDENTIFIER: Tag = Tag::new(Class::Universal, false, 6);
    pub const OBJECT_DESCRIPTOR: Tag = Tag::new(Class::Universal, false, 7);
    pub const EXTERNAL: Tag = Tag::new(Class::Universal, true, 8);
    pub const REAL: Tag = Tag::new(Class::Universal, false, 9);
    pub const ENUMERATED: Tag = Tag::new(Class::Universal, false, 10);
    pub const EMBEDDED_PDV: Tag = Tag::new(Class::Universal, false, 11);
    pub const UTF8_STRING: Tag = Tag::new(Class::Universal, false, 12);
    pub const RELATIVE_OID: Tag = Tag::new(Class::Universal, false, 13);
    pub const SEQUENCE: Tag = Tag::new(Class::Universal, true, 16);
    pub const SET: Tag = Tag::new(Class::Universal, true, 17);
    pub const NUMERIC_STRING: Tag = Tag::new(Class::Universal, false, 18);
    pub const PRINTABLE_STRING: Tag = Tag::new(Class::Universal, false, 19);
    pub const TELETEX_STRING: Tag = Tag::new(Class::Universal, false, 20);
    pub const VIDEOTEX_STRING: Tag = Tag::new(Class::Universal, false, 21);
    pub const IA5_STRING: Tag = Tag::new(Class::Universal, false, 22);
    pub const UTC_TIME: Tag = Tag::new(Class::Universal, false, 23);
    pub const GENERALIZED_TIME: Tag = Tag::new(Class::Universal, false, 24);
    pub const GRAPHIC_STRING: Tag = Tag::new(Class::Universal, false, 25);
    pub const VISIBLE_STRING: Tag = Tag::new(Class::Universal, false, 26);
    pub const GENERAL_STRING: Tag = Tag::new(Class::Universal, false, 27);
    pub const UNIVERSAL_STRING: Tag = Tag::new(Class::Universal, false, 28);
    pub const CHARACTER_STRING: Tag = Tag::new(Class::Universal, false, 29);
    pub const BMP_STRING: Tag = Tag::new(Class::Universal, false, 30);


    pub const fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self {
            class,
            is_constructed,
            tag,
        }
    }

    pub fn from_context<I: Into<u64>>(is_constructed: bool, tag: I) -> Self {
        // TODO: This is bad as it will implicitly truncate larger tags.
        // This will be fixed when tag is moved to BigInt.
        Self::new(Class::Context, is_constructed, tag.into() as usize)
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

    pub(crate) fn to_vec(self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.encode(&mut vec)?;
        Ok(vec)
    }

    pub(crate) fn decode(bytes: &[u8]) -> nom::IResult<&[u8], Self> {
        crate::decoder::parser::parse_identifier_octet(bytes)
    }

    pub(crate) fn encode<W: Write>(self, buffer: &mut W) -> Result<()> {
        let mut tag_byte = self.class as u8;
        let mut tag_number = self.tag;

        // Constructed is a single bit.
        tag_byte <<= 1;
        if self.is_constructed {
            tag_byte |= 1;
        }

        // Tag number is five bits
        tag_byte <<= 5;

        if tag_number >= 0x1f {
            tag_byte |= 0x1f;
            buffer.write(&[tag_byte])?;

            while tag_number != 0 {
                let mut encoded_number: u8 = (tag_number & 0x7f) as u8;
                tag_number >>= 7;

                // Fill the last bit unless we're at the last bit.
                if tag_number != 0 {
                    encoded_number |= 0x80;
                }

                buffer.write(&[encoded_number])?;
            }

        } else {
            tag_byte |= tag_number as u8;
            buffer.write(&[tag_byte])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Tag;

    #[test]
    fn external() {
        let mut buffer = Vec::new();
        let raw_tag = &[0b00_1_01000u8][..];
        Tag::encode(Tag::EXTERNAL, &mut buffer).unwrap();

        assert_eq!(raw_tag, &*buffer);
        assert_eq!(Tag::EXTERNAL, Tag::decode(raw_tag).unwrap().1);
    }

    #[test]
    fn builtin_tags() {
        let mut buffer = Vec::new();
        let tags = [
            Tag::EOC,
            Tag::BOOL,
            Tag::INTEGER,
            Tag::BIT_STRING,
            Tag::OCTET_STRING,
            Tag::NULL,
            Tag::OBJECT_IDENTIFIER,
            Tag::OBJECT_DESCRIPTOR,
            Tag::EXTERNAL,
            Tag::REAL,
            Tag::ENUMERATED,
            Tag::EMBEDDED_PDV,
            Tag::UTF8_STRING,
            Tag::RELATIVE_OID,
            Tag::SEQUENCE,
            Tag::SET,
            Tag::NUMERIC_STRING,
            Tag::PRINTABLE_STRING,
            Tag::TELETEX_STRING,
            Tag::VIDEOTEX_STRING,
            Tag::IA5_STRING,
            Tag::UTC_TIME,
            Tag::GENERALIZED_TIME,
            Tag::GRAPHIC_STRING,
            Tag::VISIBLE_STRING,
            Tag::GENERAL_STRING,
            Tag::UNIVERSAL_STRING,
            Tag::CHARACTER_STRING,
            Tag::BMP_STRING,
        ];

        for tag in tags.into_iter() {
            buffer.clear();
            tag.encode(&mut buffer).unwrap();

            assert_eq!(*tag, Tag::decode(&buffer).unwrap().1);
        }
    }
}
