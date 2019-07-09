//! # Abstract Syntax Notation One (ASN.1)
//! This crate is a collection of implementations for the ITU-T X.680–699 set of
//! standards, also known as ASN.1. ASN.1 is divided between two areas. The
//! notation language used to specify Protocol Data Units (PDUs), and the
//! encoding/decoding rules for encoding the specification notation.
//!
//! ## Use cases of ASN.1
//! ASN.1 is used in variety of applications, it's use is mainly in open
//! standards for large organisations as ASN.1 itself is an open standard, and
//! the notation provides a language independent way to define protocol
//! messages. The notation language also provides **extensibility** as a
//! first-class citizen, allowing standards to be iterable, without breaking
//! existing production applications. Below are some examples of areas where
//! ASN.1 is in use today.
//!
//! * **Information sharing** — X.500 Directory & LDAP
//! * **Security** — X.509 certificates, PKCS#12
//! * **Wireless communication** — LTE, 5G
//! * **RFID** — ISO 7816-4 (Organization, security and commands for interchange)
//! * **Aviation** — The Aeronautical Telecommunication Network
//!
//! ## What the `asn1` crate provides.
//! The `asn1` is a facade crate over the `asn1_core`, `asn1_der`, and
//! `asn1_notation` crates.
//!
//! * [`asn1_core`] provides definitions ASN.1 data types for handling ASN.1 data
//! as well for defining ASN.1 encoding rules.
//!
//! * [`asn1_der`] provides [`serde::{Deserialize, Serialize}`] implementations
//! for ASN.1 DER (Distingushed Encoding Rules).
//!
//! * [`asn1_notation`] provides a interface to an ASN.1 notation compiler.
//! **Note:** The notation compiler is still a work in progress, and its use
//! is **not currently recommended.**
//!
//! [`asn1_core`]: ./core/
//! [`asn1_der`]: ./der/
//! [`asn1_notation`]: ./notation/
//! [`serde::{Deserialize, Serialize}`]: docs.serde.rs/serde

pub use core::{self, *};

#[cfg(feature = "der")]
pub use der;

#[cfg(feature = "notation")]
pub use notation;
