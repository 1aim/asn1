use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typenum::marker_traits::Unsigned;

use crate::identifier::constant::*;
use crate::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename = "ASN.1#Explicit")]
pub struct Explicit<C: Class, N: Unsigned, T> {
    #[serde(skip)]
    phantom: std::marker::PhantomData<ConstIdentifier<C, N>>,
    value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Implicit<C: Class, N: Unsigned, T> {
    phantom: std::marker::PhantomData<ConstIdentifier<C, N>>,
    value: T,
}

impl<C: Class, N: Unsigned, T> Implicit<C, N, T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<C: Class, N: Unsigned, T> Explicit<C, N, T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<C: Class, N: Unsigned, T> Implicit<C, N, T> {
    const IDENTIFIER: Identifier = Identifier::new(C::CLASS, N::USIZE);
}

impl<C: Class, N: Unsigned, T> Explicit<C, N, T> {
    const IDENTIFIER: Identifier = Identifier::new(C::CLASS, N::USIZE);
}

pub struct ConstIdentifier<C: Class, N: Unsigned> {
    class: PhantomData<C>,
    tag: PhantomData<N>,
}

impl<C: Class, N: Unsigned, T: Serialize> Serialize for Implicit<C, N, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let identifier = Self::IDENTIFIER;
        serializer.serialize_newtype_struct(
            "ASN.1#Implicit",
            &(identifier.class as u8, identifier.tag, &self.value),
        )
    }
}

impl<'de, C: Class, N: Unsigned, T: Deserialize<'de>> Deserialize<'de> for Implicit<C, N, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Implicit::new(T::deserialize(deserializer)?))
    }
}

impl<C: Class, N: Unsigned, T: Serialize> Serialize for Explicit<C, N, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let identifier = Self::IDENTIFIER;
        serializer.serialize_newtype_struct(
            "ASN.1#Explicit",
            &(identifier.class as u8, identifier.tag, &self.value),
        )
    }
}
