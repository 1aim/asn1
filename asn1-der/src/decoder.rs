use std::convert::{TryFrom, TryInto};

use failure::Fallible;
use nom::*;

use crate::tag::Tag;
use crate::value::*;
use core::Class;

pub fn from_der<'a, T>(bytes: &'a [u8]) -> Fallible<T>
where
    T: TryFrom<Value<&'a [u8]>, Error = failure::Error>,
{
    let (_, value) = parse_value(bytes).unwrap();

    Ok(value.try_into()?)
}

pub fn from_der_partial<'a, T>(bytes: &'a [u8]) -> Fallible<(&'a [u8], T)>
where
    T: TryFrom<Value<&'a [u8]>, Error = failure::Error>,
{
    let (slice, value) = parse_value(bytes).unwrap();

    Ok((slice, value.try_into()?))
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
    let mut result: usize = 0;

    for byte in body {
        result <<= width;
        result |= *byte as usize;
    }

    result
}

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
    } else {
        take!(input, length)
    }
}

named!(pub(crate) parse_value<&[u8], Value<&[u8]>>, do_parse!(
    tag: parse_identifier_octet >>
    contents: parse_contents >>
    (Value::new(tag, contents))
));

#[cfg(test)]
mod tests {
    use super::*;
    use core::types::ObjectIdentifier;

    macro_rules! variant_tests {
        ($($test_fn:ident : {$($fn_name:ident ($input:expr) == $expected:expr);+;})+) => {
            $(
                $(
                    #[test]
                    fn $fn_name() {
                        let (rest, result) = $test_fn($input.into()).unwrap();
                        eprintln!("REST {:?}", rest);
                        assert_eq!($expected, result);
                    }
                )+
            )+
        }
    }

    variant_tests! {
        parse_identifier_octet: {
            universal_bool(&[0x1][..]) == Tag::new(Class::Universal, false, 1);
            private_primitive(&[0xC0][..]) == Tag::new(Class::Private, false, 0);
            context_constructed(&[0xA0][..]) == Tag::new(Class::Context, true, 0);
            private_long_constructed(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F][..])
                == Tag::new(Class::Private, true, 0x1FFFFFFFFFFFF);
        }

        parse_value: {
            primitive_bool(&[0x1, 0x1, 0xFF][..]) == Value::<&[u8]>::new(Tag::new(Class::Universal, false, 1), &[0xff]);
        }
    }

    #[test]
    fn value_to_bool() {
        let (_, yes) = parse_value([0x1, 0x1, 0xFF][..].into()).unwrap();
        let (_, no) = parse_value([0x1, 0x1, 0x00][..].into()).unwrap();

        assert!(yes.as_bool().unwrap());
        assert!(!no.as_bool().unwrap());
    }

    #[test]
    fn value_long_length() {
        let (_, value) = parse_value([0x1, 0x81, 0x2, 0xF0, 0xF0][..].into()).unwrap();

        assert_eq!(value.contents, &[0xF0, 0xF0]);
    }

    #[test]
    fn value_really_long_length() {
        let full_buffer = [0xff; 0x100];

        let mut value = vec![0x1, 0x82, 0x1, 0x0];
        value.extend_from_slice(&full_buffer);

        let (_, value) = parse_value((&*value).into()).unwrap();

        assert_eq!(value.contents, &full_buffer[..]);
    }

    #[test]
    fn value_indefinite_length() {
        let (_, value) = parse_value([0x1, 0x80, 0xf0, 0xf0, 0xf0, 0xf0, 0, 0][..].into()).unwrap();

        assert_eq!(value.contents, &[0xf0, 0xf0, 0xf0, 0xf0]);
    }

    #[test]
    fn pkcs12_to_value() {
        let _ = parse_value((&*std::fs::read("tests/data/test.p12").unwrap()).into()).unwrap();
    }

    #[test]
    fn oid_from_bytes() {
        let (_, value) =
            parse_value([0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..].into()).unwrap();
        let oid = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();

        assert_eq!(oid, value.try_into().unwrap());
    }
}
