use core::{AsnType, identifier::Identifier};

pub trait DerDecodable: AsnType {
    pub fn validate_identifier(identifier: Identifier) -> Self;
    pub fn decode(slice: &[u8]) -> Self;
}

impl DerDecodable for bool {
    pub fn decode(slice: &[u8]) -> Self;
    pub fn decode(slice: &[u8]) -> Self;
}


