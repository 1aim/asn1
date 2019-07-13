use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};


/// A representation of the `ENUMERATED` ASN.1 data type. `Enumerated` should be
/// a wrapper around an `enum` with no inner data. Using `Enumerated` will
/// produce the `ENUMERATED` encoding where as using `enum`s directly will
/// produce the `CHOICE` encoding.
pub struct Enumerated<E: Enumerable>(E);

impl<E: Enumerable> Enumerated<E> {
    /// Instantiate a new instance of `Enumerated` with an `Enumerable` variant.
    pub fn new(enumerable: E) -> Self {
        Enumerated(enumerable)
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
        serializer.serialize_newtype_struct("ASN.1#Enumerated", &self.0)
    }
}

impl<'de, E: Enumerable + Deserialize<'de>> Deserialize<'de> for Enumerated<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Enumerated(E::deserialize(deserializer)?))
    }
}

