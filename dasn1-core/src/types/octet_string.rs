use std::ops::{Deref, DerefMut};

use crate::{AsnType, Identifier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename = "ASN.1#OctetString")]
pub struct OctetString(Vec<u8>);

impl AsnType for OctetString {
    fn identifier(&self) -> Identifier {
        Identifier::OCTET_STRING
    }
}

impl OctetString {
    pub fn new() -> Self {
        Self::default()
    }

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
