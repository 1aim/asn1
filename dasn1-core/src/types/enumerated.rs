use std::{fmt, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{SeqAccess, Visitor}};

const NAME: &str = "ASN.1#Enumerated";

/// A representation of the `ENUMERATED` ASN.1 data type. `Enumerated` should be
/// a wrapper around an `enum` with no inner data. Using `Enumerated` will
/// produce the `ENUMERATED` encoding where as using `enum`s directly will
/// produce the `CHOICE` encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Enumerated<E: Enumerable>(E);

impl<E: Enumerable> Enumerated<E> {
    /// Instantiate a new instance of `Enumerated` with an `Enumerable` variant.
    pub fn new(enumerable: E) -> Self {
        Enumerated(enumerable)
    }

    /// Consumes self and returns the inner `Enumerable` variant from an instance of `Enumerated`.
    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E: Enumerable> AsRef<E> for Enumerated<E> {
    fn as_ref(&self) -> &E {
        &self.0
    }
}

impl<E: Enumerable> AsMut<E> for Enumerated<E> {
    fn as_mut(&mut self) -> &mut E {
        &mut self.0
    }
}

/// A marker trait signifying that a type is an `enum` with no data in the
/// variants. Implementing this trait on `enum`s with data or on `struct`s, and
/// using the `Enumerated` struct will produce malformed encodings.
pub trait Enumerable {}

impl<E: Enumerable + Serialize> Serialize for Enumerated<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct(NAME, &self.0)
    }
}

impl<'de, E: Enumerable + Deserialize<'de>> Deserialize<'de> for Enumerated<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = deserializer.deserialize_newtype_struct(NAME, EnumeratedVisitor::<E>::new())?;

        Ok(Enumerated::new(value))

    }
}

struct EnumeratedVisitor<T> {
    phantom: PhantomData<T>,
}

impl<T> EnumeratedVisitor<T> {
    fn new() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for EnumeratedVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a enumerable")
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        T::deserialize(de)
    }
}
