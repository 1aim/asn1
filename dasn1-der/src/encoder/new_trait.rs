use core::{AsnType, Identifier, types};

use super::Serializer;

pub fn encode_length(original_length: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(1);

    if original_length <= 127 {
        output.push(original_length as u8);
    } else {
        let mut length = original_length;
        let mut length_buffer = std::collections::VecDeque::new();

        while length != 0 {
            length_buffer.push_front((length & 0xff) as u8);
            length >>= 8;
        }

        let mut length_buffer: Vec<u8> = length_buffer.into();
        output.push(length_buffer.len() as u8 | 0x80);
        output.append(&mut length_buffer);
    }

    output
}

pub trait DerEncodable: AsnType {
    fn encode_implicit(&self, identifier: Identifier) -> Vec<u8> {
        let mut buffer = identifier.encode_der();
        let mut value = dbg!(self.encode_value());
        buffer.append(&mut encode_length(value.len()));
        buffer.append(&mut value);

        buffer
    }

    fn encode_value(&self) -> Vec<u8>;

    fn encode_der(&self) -> Vec<u8> {
        self.encode_implicit(self.identifier())
    }
}

impl DerEncodable for bool {
    fn encode_value(&self) -> Vec<u8> {
        Serializer::serialize_to_vec(self, true).unwrap().output
    }
}

impl<'a> DerEncodable for &'a str {
    fn encode_value(&self) -> Vec<u8> {
        Serializer::serialize_to_vec(self, true).unwrap().output
    }
}

macro_rules! integers {
    ($($int:ty)+) => {
        $(
            impl DerEncodable for $int {
                fn encode_value(&self) -> Vec<u8> {
                    Serializer::serialize_to_vec(self, true).unwrap().output
                }
            }
        )+
    }
}

integers! {
    u8 u16 u32 u64 u128 i8 i16 i32 i64 i128
}

impl<T: DerEncodable + serde::Serialize> DerEncodable for Vec<T> {
    fn encode_value(&self) -> Vec<u8> {
        Serializer::serialize_to_vec(self, true).unwrap().output
    }
}
impl<T: DerEncodable + serde::Serialize> DerEncodable for Option<T> {
    fn encode_implicit(&self, identifier: Identifier) -> Vec<u8> {
        self.as_ref()
            .map(|t| t.encode_implicit(identifier))
            .unwrap_or_else(Vec::new)
    }

    fn encode_value(&self) -> Vec<u8> {
        self.as_ref()
            .map(T::encode_value)
            .unwrap_or_else(Vec::new)
    }

    fn encode_der(&self) -> Vec<u8> {
        self.as_ref()
            .map(T::encode_der)
            .unwrap_or_else(Vec::new)
    }
}

impl<T: DerEncodable> DerEncodable for [T; 0] {
    fn encode_value(&self) -> Vec<u8> {
        Vec::new()
    }
}

macro_rules! arrays {
    ($($num:tt)+) => {

        $(
            impl<T: DerEncodable + serde::Serialize> DerEncodable for [T; $num] {
                fn encode_value(&self) -> Vec<u8> {
                    Serializer::serialize_to_vec(self, true).unwrap().output
                }
            }
        )+
    }
}


arrays! {
    1 2 3 4 5 6 7 8 9 10
    11 12 13 14 15 16 17 18 19 20
}


impl DerEncodable for core::Identifier {
    fn encode_der(&self) -> Vec<u8> {
        self.encode_value()
    }

    fn encode_value(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(1);

        let mut tag_byte = self.class as u8;
        let mut tag_number = self.tag;

        // Constructed is a single bit.
        tag_byte <<= 1;
        tag_byte |= match *self {
            Self::EXTERNAL |
            Self::SEQUENCE |
            Self::SET      => 1,
            _ => 0,
        };

        // Identifier number is five bits
        tag_byte <<= 5;

        if tag_number >= 0x1f {
            tag_byte |= 0x1f;
            output.push(tag_byte);

            while tag_number != 0 {
                let mut encoded_number: u8 = (tag_number & 0x7f) as u8;
                tag_number >>= 7;

                // Fill the last bit unless we're at the last bit.
                if tag_number != 0 {
                    encoded_number |= 0x80;
                }

                output.push(encoded_number);
            }
        } else {
            tag_byte |= tag_number as u8;
            output.push(tag_byte);
        }

        output
    }
}

impl DerEncodable for types::OctetString {
    fn encode_value(&self) -> Vec<u8> {
        (**self).clone()
    }
}
