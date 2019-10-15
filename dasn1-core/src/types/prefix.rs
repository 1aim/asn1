use std::{fmt, marker::PhantomData};

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use typenum::marker_traits::Unsigned;

use crate::{
    identifier::{constant::*, Class, Identifier},
    AsnType,
};

pub type Implicit<C, N, T> = ConstPrefixed<ImplicitPrefix, C, N, T>;
pub type Explicit<C, N, T> = ConstPrefixed<ExplicitPrefix, C, N, T>;

#[derive(Debug, Clone, PartialEq)]
pub struct ConstPrefixed<P: Prefix, C: ConstClass, N: Unsigned, T> {
    phantom: std::marker::PhantomData<ConstIdentifier<P, C, N>>,
    value: T,
}

impl<P: Prefix, C: ConstClass, N: Unsigned, T> From<T> for ConstPrefixed<P, C, N, T> {
    fn from(value: T) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prefixed<T> {
    prefix: Identifier,
    value: T,
}

#[derive(Debug, Clone, PartialEq)]
struct ConstIdentifier<P: Prefix, C: ConstClass, N: Unsigned> {
    prefix: PhantomData<P>,
    class: PhantomData<C>,
    tag: PhantomData<N>,
}

impl<P: Prefix, C: ConstClass, N: Unsigned> AsnType for ConstIdentifier<P, C, N> {
    fn identifier() -> Identifier {
        Identifier::new(C::CLASS, N::U32)
    }
}

impl<P: Prefix, C: ConstClass, N: Unsigned, T> ConstPrefixed<P, C, N, T> {
    const IDENTIFIER: Identifier = Identifier::new(C::CLASS, N::U32);
    const NAME: &'static str = P::NAME;

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

impl<'de, P: Prefix, C: ConstClass, N: Unsigned, T: Deserialize<'de>> Deserialize<'de>
    for ConstPrefixed<P, C, N, T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = deserializer
            .deserialize_newtype_struct(Self::NAME, PrefixVisitor::<T>::new(Self::IDENTIFIER))?;

        Ok(Self::new(value))
    }
}

impl<P: Prefix, C: ConstClass, N: Unsigned, T: Serialize> Serialize for ConstPrefixed<P, C, N, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let identifier = Self::IDENTIFIER;
        serializer.serialize_newtype_struct(
            Self::NAME,
            &(identifier.class as u8, identifier.tag, &self.value),
        )
    }
}

struct PrefixVisitor<T> {
    phantom: PhantomData<T>,
    identifier: Identifier,
}

impl<T> PrefixVisitor<T> {
    fn new(identifier: Identifier) -> Self {
        Self {
            phantom: PhantomData,
            identifier,
        }
    }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for PrefixVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a integer")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut visitor: S) -> Result<Self::Value, S::Error> {
        let class: u8 = visitor.next_element()?.unwrap();
        let tag: u32 = visitor.next_element()?.unwrap();
        let actual_identifier = Identifier::new(Class::from_u8(class), tag);

        if self.identifier != actual_identifier {
            return Err(de::Error::custom(format!(
                "{:?} != {:?}",
                self.identifier, actual_identifier
            )));
        }

        Ok(visitor
            .next_element()?
            .expect("Couldn't deserialize to inner type"))
    }
}
