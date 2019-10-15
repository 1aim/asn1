use crate::error::{Error, Result};
use serde::de::{DeserializeSeed, IntoDeserializer, SeqAccess};

use super::{Deserializer, Value};
use crate::identifier::BerIdentifier;

/// An BitString deserializer
pub(crate) struct Prefix<'a, 'de: 'a> {
    explicit: bool,
    identifier: BerIdentifier,
    sent_class: bool,
    sent_tag: bool,
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Prefix<'a, 'de> {
    pub(crate) fn new(de: &'a mut Deserializer<'de>, explicit: bool) -> Result<Self> {
        let identifier = de.peek_at_identifier()?;
        Ok(Self {
            de,
            identifier,
            sent_class: false,
            sent_tag: false,
            explicit,
        })
    }
}

impl<'a, 'de> SeqAccess<'de> for Prefix<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if !self.sent_class {
            log::trace!("Sending class {:?}.", self.identifier.class);
            self.sent_class = true;
            seed.deserialize((self.identifier.class as u8).into_deserializer())
                .map(Some)
        } else if !self.sent_tag {
            log::trace!("Sending tag '{:?}'.", self.identifier.tag);
            self.sent_tag = true;
            seed.deserialize((self.identifier.tag as usize).into_deserializer())
                .map(Some)
        } else {
            log::trace!("Deserialising inner value, explicit: {:?}", self.explicit);
            if self.explicit {
                let value = self.de.parse_value(None)?;
                seed.deserialize(&mut Deserializer::from_slice(value.contents))
                    .map(Some)
            } else {
                self.de.type_check = false;
                seed.deserialize(&mut *self.de).map(Some)
            }
        }
    }
}
