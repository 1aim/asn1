use std::convert::TryFrom;

use core::Class;
use nom::*;

use crate::{tag::Tag, value::Value};

named!(
    parse_initial_octet<Tag>,
    bits!(do_parse!(
        class: map!(take_bits!(u8, 2), Class::try_from)
            >> is_constructed: map!(take_bits!(u8, 1), is_constructed)
            >> tag: take_bits!(usize, 5)
            >> (Tag::new(class.expect("Invalid class"), is_constructed, tag))
    ))
);

named!(pub(crate) parse_identifier_octet<Tag>, do_parse!(
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

named!(
    parse_contents,
    do_parse!(length: take!(1) >> contents: apply!(take_contents, length[0]) >> (&contents))
);

named!(pub(crate) parse_value<&[u8], Value<&[u8]>>, do_parse!(
    tag: parse_identifier_octet >>
    contents: parse_contents >>
    (Value::new(tag, contents))
));

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
    // end doesn't need to be bitmasked as we know the MSB is `0`
    // (X.690 8.1.2.4.2.a).
    tag |= end as usize;

    tag
}

fn concat_bits(body: &[u8], width: u8) -> usize {
    let mut result: usize = 0;

    for byte in body {
        result <<= width;
        result |= *byte as usize;
    }

    result
}

fn take_contents(input: &[u8], length: u8) -> IResult<&[u8], &[u8]> {
    if length == 128 {
        take_until_and_consume!(input, &[0, 0][..])
    } else if length >= 127 {
        let length = length ^ 0x80;
        do_parse!(
            input,
            length: take!(length)
                >> result: value!(concat_bits(&length, 8))
                >> contents: take!(result)
                >> (contents)
        )
    } else if length == 0 {
        Ok((input, &[]))
    } else {
        take!(input, length)
    }
}

