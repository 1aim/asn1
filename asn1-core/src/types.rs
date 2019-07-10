//! This module encompasses all of the ASN.1 data types.
pub mod object_identifier;
pub mod octet_string;

pub use self::object_identifier::ObjectIdentifier;
pub use self::octet_string::OctetString;
