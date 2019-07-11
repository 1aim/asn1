//! This module encompasses the representation of the ASN.1 tags.
//!
//! Prior to the introduction of the `AUTOMATIC TAGS` construct, ASN.1
//! specifications frequently contained tags. The following subclauses describe
//! the way in which tagging was typically applied. With the introduction of
//! `AUTOMATIC TAGS`, new ASN.1 specifications need make no use of the tag
//! notation, although those modifying old notation may have to concern
//! themselves with tags. New users of the ASN.1 notation are encouraged to use
//! `AUTOMATIC TAGS` as this makes the notation more readable.
//!
//! Guidance on use of tags in new ASN.1 specifications is quite simple:
//! **DON'T USE TAGS**. Put `AUTOMATIC TAGS` in the module header, then forget
//! about tags. If you need to add new components to the `SET`, `SEQUENCE` or
//! `CHOICE` in a later version, add them to the end.
//!
//! `Universal` class tags are used only in X.680 and their use is reserved for
//! that specification to define ASN.1 data types.
//!
//! # Example
//! A frequently encountered style for the use of tags is to assign an
//! `Application` class tag precisely once in the entire specification, using it
//! to identify a type that finds wide, scattered, use within the specification.
//! An `Application` class tag is also frequently used (once only) to tag the
//! types in the outermost `CHOICE` of an application, providing identification
//! of individual messages by the `Application` class tag. The following is an
//! example use in the former case:
//! ```asn1
//! FileName ::= [APPLICATION 8] SEQUENCE {
//!     directoryName VisibleString,
//!     directoryRelativeFileName VisibleString
//! }
//! ```
//!
//! # Example
//! `Context`-specific tagging is frequently applied in an algorithmic manner to
//! all components of a `SET`, `SEQUENCE`, or `CHOICE`. **Note**, however, that
//! the `AUTOMATIC TAGS` facility does this easily for you.
//! ```asn1
//! CustomerRecord ::= SET {
//!     name [0] VisibleString,
//!     mailingAddress [1] VisibleString,
//!     accountNumber [2] INTEGER,
//!     balanceDue [3] INTEGER -- in cents --}
//!
//! CustomerAttribute ::= CHOICE {
//!     name [0] VisibleString,
//!     mailingAddress [1] VisibleString,
//!     accountNumber [2] INTEGER,
//!     balanceDue [3] INTEGER -- in cents --}
//! ```
//!
//! # Example
//! `Private` class tagging should normally not be used in internationally
//! standardized specifications (although this cannot be prohibited).
//! Applications produced by an enterprise will normally use `Application` and
//! `Context`-specific tag classes. There may be occasional cases, however,
//! where an enterprise-specific specification seeks to extend an
//! internationally standardized specification, and in this case use of
//! `Private` class tags may give some benefits in partially protecting the
//! enterprise-specific specification from changes to the internationally
//! standardized specification.
//! ```asn1
//! AcmeBadgeNumber ::= [PRIVATE 2] INTEGER
//! badgeNumber AcmeBadgeNumber ::= 2345
//! ```
//!
//! # Example
//! Textual use of `IMPLICIT` with every tag is generally found only in older
//! specifications. BER produces a less compact representation when explicit
//! tagging is used than when implicit tagging is used. PER produces the same
//! compact encoding in both cases. With BER and explicit tagging, there is more
//! visibility of the underlying type (`INTEGER`, `REAL`, `BOOLEAN`, etc.) in
//! the encoded data. These guidelines use implicit tagging in the examples
//! whenever it is legal to do so. This may, depending on the encoding rules,
//! result in a compact representation, which is highly desirable in some
//! applications. In other applications compactness may be less important than,
//! for example, the ability to carry out strong type-checking. In the latter
//! case explicit tagging can be used.
//! ```asn1
//! CustomerRecord ::= SET {
//!     name [0] IMPLICIT VisibleString,
//!     mailingAddress [1] IMPLICIT VisibleString,
//!     accountNumber [2] IMPLICIT INTEGER,
//!     balanceDue [3] IMPLICIT INTEGER -- in cents --}
//!
//! CustomerAttribute ::= CHOICE {
//!     name [0] IMPLICIT VisibleString,
//!     mailingAddress [1] IMPLICIT VisibleString,
//!     accountNumber [2] IMPLICIT INTEGER,
//!     balanceDue [3] IMPLICIT INTEGER -- in cents --}
//! ```

/// The class of an `Identifier`.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Class {
    Universal = 0,
    Application,
    Context,
    Private,
}

/// An abstract representation of the identifier octets used in BER, CER, and
/// DER to identify an encoded value.
///
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Identifier {
    /// The class of the Identifier.
    pub class: Class,
    /// Whether the Identifier uses constructed or primitive encoding.
    pub is_constructed: bool,
    /// The specific tag number.
    pub tag: usize,
}

impl Identifier {
    pub const EOC: Identifier = Identifier::new(Class::Universal, false, 0);
    pub const BOOL: Identifier = Identifier::new(Class::Universal, false, 1);
    pub const INTEGER: Identifier = Identifier::new(Class::Universal, false, 2);
    pub const BIT_STRING: Identifier = Identifier::new(Class::Universal, false, 3);
    pub const OCTET_STRING: Identifier = Identifier::new(Class::Universal, false, 4);
    pub const NULL: Identifier = Identifier::new(Class::Universal, false, 5);
    pub const OBJECT_IDENTIFIER: Identifier = Identifier::new(Class::Universal, false, 6);
    pub const OBJECT_DESCRIPTOR: Identifier = Identifier::new(Class::Universal, false, 7);
    pub const EXTERNAL: Identifier = Identifier::new(Class::Universal, true, 8);
    pub const REAL: Identifier = Identifier::new(Class::Universal, false, 9);
    pub const ENUMERATED: Identifier = Identifier::new(Class::Universal, false, 10);
    pub const EMBEDDED_PDV: Identifier = Identifier::new(Class::Universal, false, 11);
    pub const UTF8_STRING: Identifier = Identifier::new(Class::Universal, false, 12);
    pub const RELATIVE_OID: Identifier = Identifier::new(Class::Universal, false, 13);
    pub const SEQUENCE: Identifier = Identifier::new(Class::Universal, true, 16);
    pub const SET: Identifier = Identifier::new(Class::Universal, true, 17);
    pub const NUMERIC_STRING: Identifier = Identifier::new(Class::Universal, false, 18);
    pub const PRINTABLE_STRING: Identifier = Identifier::new(Class::Universal, false, 19);
    pub const TELETEX_STRING: Identifier = Identifier::new(Class::Universal, false, 20);
    pub const VIDEOTEX_STRING: Identifier = Identifier::new(Class::Universal, false, 21);
    pub const IA5_STRING: Identifier = Identifier::new(Class::Universal, false, 22);
    pub const UTC_TIME: Identifier = Identifier::new(Class::Universal, false, 23);
    pub const GENERALIZED_TIME: Identifier = Identifier::new(Class::Universal, false, 24);
    pub const GRAPHIC_STRING: Identifier = Identifier::new(Class::Universal, false, 25);
    pub const VISIBLE_STRING: Identifier = Identifier::new(Class::Universal, false, 26);
    pub const GENERAL_STRING: Identifier = Identifier::new(Class::Universal, false, 27);
    pub const UNIVERSAL_STRING: Identifier = Identifier::new(Class::Universal, false, 28);
    pub const CHARACTER_STRING: Identifier = Identifier::new(Class::Universal, false, 29);
    pub const BMP_STRING: Identifier = Identifier::new(Class::Universal, false, 30);

    /// Instantiates a new instance of `Identifier` with the arguments provided.
    pub const fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self {
            class,
            is_constructed,
            tag,
        }
    }

    /// Instantiates a instance of `Identifier` with the `Context` class.
    pub fn from_context<I: Into<u64>>(is_constructed: bool, tag: I) -> Self {
        // TODO: This is bad as it will implicitly truncate larger tags.
        // This will be fixed when tag is moved to BigInt.
        Self::new(Class::Context, is_constructed, tag.into() as usize)
    }

    /// Overrides the current `Identifier`'s tag number.
    pub fn set_tag(mut self, tag: usize) -> Self {
        self.tag = tag;
        self
    }

    /// Returns the length of the `Identifier` in bytes.
    pub fn len(&self) -> usize {
        if self.tag > 0x1f {
            2
        } else {
            1
        }
    }
}

impl From<u8> for Class {
    fn from(value: u8) -> Self {
        match value {
            0 => Class::Universal,
            1 => Class::Application,
            2 => Class::Context,
            3 => Class::Private,
            _ => panic!("Impossible Class"),
        }
    }
}
