#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Class {
    Universal = 0,
    Application,
    Context,
    Private,
}

/// An abstract representation of the identifier octets used in BER, CER, and
/// DER to identify .
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Identifier {
    pub class: Class,
    pub is_constructed: bool,
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

    pub const fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self {
            class,
            is_constructed,
            tag,
        }
    }

    pub fn from_context<I: Into<u64>>(is_constructed: bool, tag: I) -> Self {
        // TODO: This is bad as it will implicitly truncate larger tags.
        // This will be fixed when tag is moved to BigInt.
        Self::new(Class::Context, is_constructed, tag.into() as usize)
    }

    pub fn set_tag(mut self, tag: usize) -> Self {
        self.tag = tag;
        self
    }

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
