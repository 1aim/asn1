use core::identifier::Identifier;
use serde::{
    de::{self, SeqAccess, Visitor, Deserializer},
    forward_to_deserialize_any,
};

use crate::error::{Error, Result};

pub(crate) struct IdentifierDeserializer<'a, 'de> {
    identifier: Option<Identifier>,
    de: &'a mut super::Deserializer<'de>,
}

impl<'a, 'de: 'a> IdentifierDeserializer<'a, 'de> {
    pub fn new(identifier: Option<Identifier>, de: &'a mut super::Deserializer<'de>)
        -> Self
    {
        Self {
            identifier,
            de
        }
    }

    pub fn check_and_deserialize<V>(&mut self, identifier: Identifier, visitor: V)
        -> Result<V::Value>
        where V: Visitor<'de>
    {
        log::trace!("Comparing {:?} == {:?}", self.identifier, identifier);
        if self.identifier.map(|i| i == identifier).unwrap_or(false) {
            log::trace!("Matched identifier");
            self.de.deserialize_any(visitor)
        } else {
            panic!();
            log::trace!("Didn't match identifier");
            visitor.visit_none()
        }
    }
}

impl<'a, 'de: 'a> de::Deserializer<'de> for &'a mut IdentifierDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value> {
        unimplemented!()
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::BOOL, visitor)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::INTEGER, visitor)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
}

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i8(visitor)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::REAL, visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_f32(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::UNIVERSAL_STRING, visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_char(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_char(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::OCTET_STRING, visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        // TODO Maybe support recursive option types.
        unimplemented!()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &str, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::NULL, visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        mut self,
        name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        let identifier = match name {
            "ASN.1#OctetString" => Identifier::OCTET_STRING,
            "ASN.1#ObjectIdentifier" => Identifier::OBJECT_IDENTIFIER,
            "ASN.1#BitString" => Identifier::BIT_STRING,
            "ASN.1#Enumerated" => Identifier::ENUMERATED,
            "ASN.1#Implicit" => unimplemented!(),
            "ASN.1#Explicit" => unimplemented!(),
            _ => Identifier::SEQUENCE,
        };

        self.check_and_deserialize(identifier, visitor)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &str,
        fields: &[&str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.check_and_deserialize(Identifier::SEQUENCE, visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_and_deserialize(Identifier::NULL, visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_and_deserialize(Identifier::OCTET_STRING, visitor)
    }

    forward_to_deserialize_any! {
        ignored_any identifier
    }
}

