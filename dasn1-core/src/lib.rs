//! `asn1_core` encapsulates all the data types defined in the ASN.1
//! specifications.

#![cfg_attr(test, deny(missing_docs))]
pub mod identifier;
pub mod types;

pub use identifier::Identifier;

use self::identifier::TagEncoding;

pub trait AsnType {
    fn identifier() -> Identifier;
    // Exists because choice needs to know it's variant to be encoded.
    fn choice_identifier(&self) -> Option<Identifier> {
        None
    }

    fn tag_encoding(&self) -> TagEncoding {
        TagEncoding::Untagged
    }
}

impl<'a, T: AsnType> AsnType for &'a T {
    fn identifier() -> Identifier {
        T::identifier()
    }
}

impl AsnType for str {
    fn identifier() -> Identifier {
        Identifier::UNIVERSAL_STRING
    }
}

impl AsnType for String {
    fn identifier() -> Identifier {
        <str as AsnType>::identifier()
    }
}

impl AsnType for bool {
    fn identifier() -> Identifier {
        Identifier::BOOL
    }
}

impl AsnType for () {
    fn identifier() -> Identifier {
        Identifier::NULL
    }
}

impl<T: AsnType> AsnType for Option<T> {
    fn identifier() -> Identifier {
        T::identifier()
    }

    fn tag_encoding(&self) -> TagEncoding {
        match self {
            Some(inner) => inner.tag_encoding(),
            None => TagEncoding::Untagged,
        }
    }
}

macro_rules! integers {
    ($($num:ty)+) => {
        $(
            impl AsnType for $num {
                fn identifier() -> Identifier {
                    Identifier::INTEGER
                }
            }
        )+
    }
}

integers!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

impl<T> AsnType for Vec<T> {
    fn identifier() -> Identifier {
        Identifier::SEQUENCE
    }
}

impl<T: AsnType> AsnType for [T] {
    fn identifier() -> Identifier {
        T::identifier()
    }
}

macro_rules! arrays {
    ($($num:tt)+) => {
        $(
            impl<T> AsnType for [T; $num] {
                fn identifier() -> Identifier {
                    Identifier::SEQUENCE
                }
            }
        )+
    }
}

arrays! {
    0 1 2 3 4 5 6 7 8 9 10
    11 12 13 14 15 16 17 18 19 20
}
