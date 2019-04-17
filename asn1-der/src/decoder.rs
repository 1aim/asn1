use std::convert::TryFrom;

use nom::*;

use core::{Decode, Decoder as Super, Result, Class};

#[derive(Copy, Clone, Debug, Default)]
pub struct Decoder;

impl Super for Decoder {
    const CANONICAL: bool = true;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    class: Class,
    is_constructed: bool,
    tag: usize,
}

impl Tag {
    pub fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self { class, is_constructed, tag }
    }
}

fn is_constructed(byte: u8) -> bool {
    byte != 0
}

fn is_part_of_octet(byte: u8) -> bool {
    byte & 0x80 != 0
}

fn parse_tag(body: &[u8], end: u8) -> usize {
    let mut tag = 0;

    for byte in body {
        tag <<= 7;
        tag |= (byte & 0x7F) as usize;
    }

    tag <<= 7;
    // end doesn't need to be bitmasked as we know the MSB is `0` (8.1.2.4.2.a).
    tag |= end as usize;

    tag
}

named!(parse_identifier_octet<Tag>,
    do_parse!(
        class: map!(bits!(take_bits!(u8, 2)), Class::try_from) >>
        is_constructed: map!(bits!(take_bits!(u8, 1)), is_constructed) >>
        tag: alt!(
            do_parse!(
                bits!(tag_bits!(u8, 5, 0b11111)) >>
                body: take_while!(is_part_of_octet) >>
                end: take!(1) >>
                result: value!(parse_tag(body, end[0])) >>
                (result)
            ) |
            bits!(take_bits!(usize, 5))
        ) >>
        tag: bits!(take_bits!(usize, 5)) >>

        (Tag {
            class: class.unwrap(),
            is_constructed,
            tag,
        })
    )
);

impl Decoder {
    pub fn from_bytes<T>(bytes: &'static [u8]) -> Result<T> {
        let (rest, tag) = parse_identifier_octet(bytes)?;
        unimplemented!()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_tag() {
        let expected = Tag::new(Class::Universal, false, 1);
        let (_, result) = parse_identifier_octet(&[0b00_0_00001]).unwrap();

        assert_eq!(expected, result)
    }
}
