use std::convert::TryFrom;

use nom::*;
use nom::types::CompleteByteSlice;

use core::{Decoder as Super, Result, Class};
use crate::value::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Decoder;

impl Super for Decoder {
    const CANONICAL: bool = true;
}

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
    let mut result = 0;

    for byte in body {
        result <<= width;
        result |= *byte as usize;
    }

    result
}

named!(parse_initial_octet<CompleteByteSlice, Tag>, bits!(do_parse!(
    class: map!(take_bits!(u8, 2), Class::try_from) >>
    is_constructed: map!(take_bits!(u8, 1), is_constructed) >>
    tag: take_bits!(usize, 5) >>
    (Tag::new(class.expect("Invalid class"), is_constructed, tag))
)));

named!(parse_identifier_octet<CompleteByteSlice, Tag>, do_parse!(
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

named!(parse_contents<CompleteByteSlice, &[u8]>, do_parse!(
    length: take!(1) >>
    contents: apply!(take_contents, length[0]) >>
    (&contents)
));

fn take_contents(input: CompleteByteSlice, length: u8) -> IResult<CompleteByteSlice, CompleteByteSlice> {
    if length == 128 {
        take_until_and_consume!(input, &[0, 0][..])
    } else if length >= 127 {
        do_parse!(input,
            length: take!(length) >>
            result: value!(concat_bits(&length, 8)) >>
            contents: take!(result) >>
            (contents)
        )
    } else {
        take!(input, length)
    }
}

named!(parse_value<CompleteByteSlice, Value>, do_parse!(
    tag: parse_identifier_octet >>
    contents: parse_contents >>
    (Value::new(tag, contents))
));

pub fn from_der<'a>(bytes: &'a [u8]) -> Result<Value<'a>> {
    let bytes = CompleteByteSlice::from(bytes);
    let (_, value) = parse_value(bytes).unwrap();

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! variant_tests {
        ($($test_fn:ident : {$($fn_name:ident ($input:expr) == $expected:expr);+;})+) => {
            $(
                $(
                    #[test]
                    fn $fn_name() {
                        let (rest, result) = $test_fn($input.into()).unwrap();
                        println!("REST {:?}", rest);
                        assert_eq!($expected, result);
                    }
                )+
            )+
        }
    }

    variant_tests! {
        parse_identifier_octet: {
            universal_bool([0x1][..]) == Tag::new(Class::Universal, false, 1);
            private_primitive([0xC0][..]) == Tag::new(Class::Private, false, 0);
            context_constructed([0xA0][..]) == Tag::new(Class::Context, true, 0);
            private_long_constructed([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F][..])
                == Tag::new(Class::Private, true, 0x1FFFFFFFFFFFF);
        }

        parse_value: {
            primitive_bool([0x1, 0x1, 0xFF][..]) == Value::new(Tag::new(Class::Universal, false, 1), &[0xff]);
        }
    }

    #[test]
    fn value_to_bool() {
        let (_, yes) = parse_value([0x1, 0x1, 0xFF][..].into()).unwrap();
        let (_, no) = parse_value([0x1, 0x1, 0x00][..].into()).unwrap();

        assert!(yes.as_bool().unwrap());
        assert!(!no.as_bool().unwrap());
    }
}
