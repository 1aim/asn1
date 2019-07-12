use crate::error::{Error, Result};
use serde::de::{DeserializeSeed, IntoDeserializer, SeqAccess};

/// An BitString deserializer
pub(crate) struct BitString<'de> {
    contents: std::iter::Cloned<std::slice::Iter<'de, u8>>,
}

impl<'de> BitString<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        let mut contents = data.into_iter().cloned();
        contents.next();

        Self { contents }
    }
}

impl<'de> SeqAccess<'de> for BitString<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        log::trace!("Creating BIT STRING element seed.");
        match self.contents.next() {
            Some(value) => seed.deserialize(value.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }
}
