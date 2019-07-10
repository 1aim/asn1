use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

/// A representation of the `OCTET STRING` ASN.1 data type.  Use an
/// `OCTET STRING` type to model binary data whose format and length are
/// unspecified, or specified elsewhere, and whose length in bits is a multiple
/// of eight.
/// # Example
/// ```asn1
/// G4FacsimileImage ::= OCTET STRING
/// -- a sequence of octets conforming to Rec. ITU-T T.5 and CCITT Rec. T.6
/// image G4FacsimileImage ::= '3FE2EBAD471005'H
/// ```
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "ASN.1#OctetString")]
pub struct OctetString(Vec<u8>);

impl OctetString {
    /// Instantiate an empty instance of `OctetString`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes `self` and returns the inner `Vec<u8>`.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for OctetString {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl AsRef<[u8]> for OctetString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for OctetString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OctetString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
