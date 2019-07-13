use std::fmt;

use num_bigint::BigInt;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{Error, SeqAccess, Visitor},
};

pub struct Integer(BigInt);

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("ASN.1#Integer", &self.0.to_signed_bytes_be())
    }
}

struct IntegerVisitor;

impl<'de> Visitor<'de> for IntegerVisitor {
    type Value = BigInt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bit string")
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


impl<'de> Deserialize<'de> for Integer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bigint = deserializer.deserialize_newtype_struct("ASN.1#BitString", IntegerVisitor)?;
        Ok(Integer(bigint))
    }
}
