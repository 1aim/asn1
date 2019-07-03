use core::identifier::{Class, Identifier};
use nom::IResult;

use super::Value;

pub(crate) fn parse_value(input: &[u8]) -> IResult<&[u8], Value> {
    let (input, identifier)  = parse_identifier_octet(input)?;
    let (input, contents)  = parse_contents(input)?;

    Ok((input, Value::new(identifier, contents)))
}

fn parse_contents(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, length) = nom::bytes::streaming::take(1usize)(input)?;
    take_contents(input, length[0])
}

pub(crate) fn parse_identifier_octet(input: &[u8]) -> IResult<&[u8], Identifier> {
    let (input, identifier) = parse_initial_octet(input)?;

    let (input, tag) = if identifier.tag >= 0x1f {
        let (input, body) = nom::bytes::streaming::take_while(|i| i & 0x80 != 0)(input)?;
        let (input, end) = nom::bytes::streaming::take(1usize)(input)?;

        (input, parse_tag(body, end[0]))
    } else {
        (input, identifier.tag)
    };

    Ok((input, identifier.set_tag(tag)))
}

fn parse_initial_octet(input: &[u8]) -> IResult<&[u8], Identifier> {
    let (input, octet) = nom::bytes::streaming::take(1usize)(input)?;
    let initial_octet = octet[0];

    let class_bits = (initial_octet & 0xC0) >> 6;
    let class = Class::from(class_bits);
    let constructed = (initial_octet & 0x20) != 0;
    let tag = (initial_octet & 0x1f) as usize;

    Ok((input, Identifier::new(class, constructed, tag)))
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
    if length == 0x80 {
        const EOC_OCTET: &[u8] = &[0, 0];
        let (input, contents) = nom::bytes::streaming::take_until(EOC_OCTET)(input)?;
        let (input, _) = nom::bytes::streaming::tag(EOC_OCTET)(input)?;

        Ok((input, contents))
    } else if length >= 0x7f {
        let length = length ^ 0x80;
        let (input, length_slice) = nom::bytes::streaming::take(length)(input)?;
        let length = concat_bits(&length_slice, 8);
        nom::bytes::streaming::take(length)(input)
    } else if length == 0 {
        Ok((input, &[]))
    } else {
        nom::bytes::streaming::take(length)(input)
    }
}

