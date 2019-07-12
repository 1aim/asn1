use crate::error::{Error, Result};
use serde::de::{DeserializeSeed, SeqAccess};

/// An OctetString deserializer
pub(crate) struct OctetString<'de> {
    contents: &'de [u8],
}

impl<'de> OctetString<'de> {
    pub fn new(contents: &'de [u8]) -> Self {
        Self { contents }
    }
}

impl<'de> SeqAccess<'de> for OctetString<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        use serde::de::value::SeqDeserializer;
        seed.deserialize(SeqDeserializer::new(self.contents.into_iter().cloned()))
            .map(Some)
    }
}
