//! Provides a data representation of the ASN.1 BER Identifier octets.

use core::identifier::{Class, Identifier};

/// A wrapper around `core::Identifier` except it also contains whether the tag
/// is using constructed or primitive encoding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BerIdentifier {
    /// The ASN.1 tag.
    pub identifier: Identifier,
    /// Whether a type is using `constructed` or `primitive` encoding.
    pub is_constructed: bool,
}

impl BerIdentifier {
    /// Instantiates a new instance of `BerIdentifier` from its components.
    pub fn new(class: Class, is_constructed: bool, tag: u32) -> Self {
        Self {
            identifier: Identifier::new(class, tag),
            is_constructed,
        }
    }

    /// Instantiates a new tag from `self` with `tag` overwritten.
    pub fn tag(self, tag: u32) -> Self {
        Self {
            identifier: self.identifier.set_tag(tag),
            is_constructed: self.is_constructed,
        }
    }
}

impl std::ops::Deref for BerIdentifier {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self.identifier
    }
}

impl From<Identifier> for BerIdentifier {
    fn from(identifier: Identifier) -> Self {
        Self {
            identifier,
            is_constructed: match identifier {
                Identifier::SEQUENCE | Identifier::SET | Identifier::EXTERNAL => true,
                _ => false,
            },
        }
    }
}
