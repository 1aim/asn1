use core::{Identifier, AsnType, types};
use nom::IResult;
use num_traits::ToPrimitive;
use num_bigint::BigInt;

use super::parser::{parse_contents, parse_identifier_octet};
use crate::error::{Error, Result};

pub trait DerDecodable: AsnType + Sized {
    fn parse_value(input: &[u8]) -> Result<Self>;

    fn parse_implicit(input: &[u8], identifier: Identifier) -> Result<(&[u8], Self)> {
        let (input, identifier) = parse_identifier_octet(input)?;
        if identifier != identifier {
            return Err(Error::IncorrectType {
                expected: Self::identifier(),
                actual: identifier.identifier,
            });
        }

        let (input, contents) = parse_contents(input)?;
        let value = Self::parse_value(contents)?;

        Ok((input, value))
    }

    /// Generates a DER encoding for the respective type. It is recommended to
    /// **NOT** override this method, as this could lead to incorrect
    /// DER parsing.
    fn parse_der(input: &[u8]) -> Result<(&[u8], Self)> {
        Self::parse_implicit(input, Self::identifier())
    }
}

impl DerDecodable for bool {
    fn parse_value(input: &[u8]) -> Result<Self> {
        input
            .first()
            .map(|&v| v == 0xff)
            .ok_or_else(|| Error::IncorrectLength(String::from("bool")))
    }
}

impl DerDecodable for String {
    fn parse_value(input: &[u8]) -> Result<Self> {
        String::from_utf8(input.to_owned())
            .map_err(|_| Error::Parser(String::from("Invalid UTF-8")))
    }
}

impl<T: DerDecodable> DerDecodable for Option<T> {
    fn parse_value(input: &[u8]) -> Result<Self> {
        Ok(Some(T::parse_value(input)?))
    }

    fn parse_implicit(input: &[u8], identifier: Identifier) -> Result<(&[u8], Self)> {
        match T::parse_implicit(input, identifier) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err(Error::IncorrectType { .. }) => Ok((input,  None)),
            Err(error) => Err(error.into()),
        }
    }
}

impl<T: DerDecodable> DerDecodable for Vec<T> {
    fn parse_value(mut input: &[u8]) -> Result<Self> {
        let mut vec = Vec::new();
        while let Ok((new_in, value)) = T::parse_der(input) {
            vec.push(value);
            input = new_in;
        }

        Ok(vec)
    }
}

impl DerDecodable for types::OctetString {
    fn parse_value(input: &[u8]) -> Result<Self> {
        Ok(Self::from(input.to_owned()))
    }
}

macro_rules! integers {
    ($($int:ty : $method:ident),+) => {
        $(
            impl DerDecodable for $int {
                fn parse_value(input: &[u8]) -> Result<Self> {
                    BigInt::from_signed_bytes_be(input)
                        .$method()
                        .ok_or_else(|| Error::IntegerOverflow(stringify!($int).to_string()))
                }
            }
        )+
    }
}

integers! {
    u8: to_u8,
    u16: to_u16,
    u32: to_u32,
    u64: to_u64,
    u128: to_u128,
    i8: to_i8,
    i16: to_i16,
    i32: to_i32,
    i64: to_i64,
    i128: to_i128
}
