use std::fmt;

use num_bigint::BigInt;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{Error, SeqAccess, Visitor},
};

/// A representation of the `INTEGER` ASN.1 data type. `Integer` is a wrapper
/// around the `num_bigint::BigInt` type. Please refer to the [`BigInt`]
/// documentation for using `Integer` in Rust.
///
/// [`BigInt`]: https://docs.rs/num-bigint/0.2.2/num_bigint/struct.BigInt.html
pub struct Integer(BigInt);

impl Integer {
    pub fn new(big: BigInt) -> Self {
        Self(big)
    }

    pub fn into_inner(self) -> BigInt {
        self.0
    }
}

struct IntegerVisitor;

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("ASN.1#Integer", &self.0.to_signed_bytes_be())
    }
}

impl<'de> Deserialize<'de> for Integer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bigint = deserializer.deserialize_newtype_struct("ASN.1#Integer", IntegerVisitor)?;
        Ok(Integer(bigint))
    }
}

macro_rules! integers {
    ($($int:ty)+) => {
        $(
            impl From<$int> for Integer {
                fn from(value: $int) -> Self {
                    Integer(BigInt::from(value))
                }
            }
        )+
    }
}

integers!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

impl<'de> Visitor<'de> for IntegerVisitor {
    type Value = BigInt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a integer")
    }

    fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(BigInt::from_signed_bytes_be(v))
    }

    fn visit_borrowed_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(BigInt::from_signed_bytes_be(v))
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut visitor: S) -> Result<Self::Value, S::Error> {
        let mut values = Vec::new();
        while let Some(value) = visitor.next_element()? {
            values.push(value);
        }

        Ok(BigInt::from_signed_bytes_be(&values))
    }
}
