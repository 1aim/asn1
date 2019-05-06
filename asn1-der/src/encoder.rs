use std::io::Write;

use crate::Value;
use crate::value::Tag;

pub fn to_der<A: AsRef<[u8]>, I: Into<Value<A>>>(value: I) -> Vec<u8> {
    let value = value.into();
    let mut buffer = Vec::with_capacity(value.len());

    encode_tag(value.tag, &mut buffer);
    encode_contents(value.contents.as_ref(), &mut buffer);

    buffer
}

fn encode_tag(tag: Tag, buffer: &mut Vec<u8>) {
    let mut tag_byte = tag.class as u8;
    let mut tag_number = tag.tag;

    // Constructed is a single bit.
    tag_byte <<= 1;
    if tag.is_constructed {
        tag_byte |= 1;
    }

    // Tag number is five bits, plus the the constructed or primitive bit.
    tag_byte <<= 6;
    if tag_number >= 0x1f {
        tag_byte |= 0x1f;
        buffer.push(tag_byte);

        while tag_number != 0 {
            let mut encoded_number: u8 = (tag_number & 0x7f) as u8;
            tag_number >>= 7;

            // Fill the last bit unless we're at the last bit.
            if tag_number != 0 {
                encoded_number |= 0x80;
            }

            buffer.push(encoded_number);
        }


    } else {
        tag_byte |= tag_number as u8;
        buffer.push(tag_byte)
    }
}

fn encode_contents(contents: &[u8], buffer: &mut Vec<u8>) {
    if contents.len() <= 127 {
        buffer.push(contents.len() as u8);
    } else {
        let mut length = contents.len();
        let mut length_buffer = Vec::new();

        while length != 0 {
            length_buffer.push((length & 0xff) as u8);
            length >>= 8;
        }

        buffer.push(length_buffer.len() as u8 | 0x80);
        buffer.append(&mut length_buffer);
    }

    buffer.extend_from_slice(contents);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool() {
        assert_eq!(to_der(true), &[1, 1, 255]);
        assert_eq!(to_der(false), &[1, 1, 0]);
    }
}

