use crate::error::{Error, Result};
use serde::de::{DeserializeSeed, SeqAccess, value::SeqDeserializer};

/// An ObjectIdentifier deserializer
pub(crate) struct ObjectIdentifier<'de> {
    contents: &'de [u8],
}

impl<'de> ObjectIdentifier<'de> {
    pub fn new(contents: &'de [u8]) -> Self {
        Self { contents }
    }
}

impl<'de> SeqAccess<'de> for ObjectIdentifier<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        let (input, root_octets) = super::parser::parse_encoded_number(self.contents)?;
        let second = (root_octets % 40) as u128;
        let first = ((root_octets as u128 - second) / 40) as u128;
        let mut buffer = vec![first, second];

        let mut input = input;
        while !input.is_empty() {
            let (new_input, number) = super::parser::parse_encoded_number(input)?;
            input = new_input;
            buffer.push(number as u128);
        }

        seed.deserialize(SeqDeserializer::new(buffer.into_iter())).map(Some)
    }
}


