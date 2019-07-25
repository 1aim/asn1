use num_traits::ToPrimitive;
use serde::de::{value::SeqDeserializer, DeserializeSeed, SeqAccess};

use crate::error::{Error, Result};

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
        let second = (&root_octets % 40u8).to_u32().expect("Second root component greater than `u32`");
        let first = ((root_octets - second) / 40u8).to_u32().expect("first root component greater than `u32`");
        let mut buffer = vec![first, second];

        let mut input = input;
        while !input.is_empty() {
            let (new_input, number) = super::parser::parse_encoded_number(input)?;
            input = new_input;
            buffer.push(number.to_u32().expect("sub component greater than `u32`"));
        }

        seed.deserialize(SeqDeserializer::new(buffer.into_iter()))
            .map(Some)
    }
}
