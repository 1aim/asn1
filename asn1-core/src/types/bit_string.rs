use std::{fmt, error::Error};

use bit_vec::BitVec;
use serde::{
    self,
    Deserialize,
    de::{
        Deserializer,
        SeqAccess,
        Visitor,
    },
    Serialize,
    Serializer
};


#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename="ASN.1#BitString")]
pub struct BitString(BitVec);

impl BitString {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(input: &[u8]) -> Self {
        Self(BitVec::from_bytes(input))
    }
}

struct BitStringVisitor;

impl<'de> Visitor<'de> for BitStringVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bit string")
    }

    fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
        unimplemented!()
    }

    fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.to_vec())
    }

    fn visit_borrowed_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.to_vec())
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut visitor: S) -> Result<Self::Value, S::Error> {
        let mut values = Vec::new();
        while let Some(value) = visitor.next_element()? {
            values.push(value);
        }

        Ok(values)
    }
}

impl Serialize for BitString {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = self.0.to_bytes();

        if let Some(last) = bytes.last() {
            let zeroes = last.trailing_zeros();
            bytes.insert(0, zeroes as u8);
        } else {
            // If there is no last, then the vec is empty and we put in a single
            // zero octet. (X.690 8.6.2.3).
            bytes.push(0);
        }

        serializer.serialize_newtype_struct("ASN.1#BitString", &bytes)
    }
}
