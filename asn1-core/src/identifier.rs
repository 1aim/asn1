#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Class {
    Universal = 0,
    Application,
    Context,
    Private,
}

impl Class {
    /// Instantiate a `Class` from a u8.
    ///
    /// # Panics
    /// If `value` is greater than 3.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Class::Universal,
            1 => Class::Application,
            2 => Class::Context,
            3 => Class::Private,
            num => panic!("'{}' is not a valid class of tag.", num),
        }
    }

    pub fn is_universal(self) -> bool {
        self == Class::Universal
    }
}

/// An abstract representation of the identifier octets used in BER, CER, and
/// DER to identify .
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Identifier {
    pub class: Class,
    pub tag: usize,
}

macro_rules! consts {
    ($($name:ident = $value:expr),+) => {
        $(
            pub const $name: Identifier = Identifier::new(Class::Universal, $value);
        )+
    }
}

impl Identifier {
    consts! {
        EOC = 0,
        BOOL = 1,
        INTEGER = 2,
        BIT_STRING = 3,
        OCTET_STRING = 4,
        NULL = 5,
        OBJECT_IDENTIFIER = 6,
        OBJECT_DESCRIPTOR = 7,
        EXTERNAL = 8,
        REAL = 9,
        ENUMERATED = 10,
        EMBEDDED_PDV = 11,
        UTF8_STRING = 12,
        RELATIVE_OID = 13,
        SEQUENCE = 16,
        SET = 17,
        NUMERIC_STRING = 18,
        PRINTABLE_STRING = 19,
        TELETEX_STRING = 20,
        VIDEOTEX_STRING = 21,
        IA5_STRING = 22,
        UTC_TIME = 23,
        GENERALIZED_TIME = 24,
        GRAPHIC_STRING = 25,
        VISIBLE_STRING = 26,
        GENERAL_STRING = 27,
        UNIVERSAL_STRING = 28,
        CHARACTER_STRING = 29,
        BMP_STRING = 30
    }

    pub const fn new(class: Class, tag: usize) -> Self {
        Self {
            class,
            tag,
        }
    }

    pub fn from_context<I: Into<u64>>(tag: I) -> Self {
        // TODO: This is bad as it will implicitly truncate larger tags.
        // This will be fixed when tag is moved to BigInt.
        Self::new(Class::Context, tag.into() as usize)
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

pub mod constant {
    pub trait Prefix: Copy + Clone + Ord + PartialOrd + Eq + PartialEq + std::fmt::Debug {
        const NAME: &'static str;
    }

    pub trait ConstClass: Copy + Clone + Ord + PartialOrd + Eq + PartialEq + std::fmt::Debug {
        const CLASS: super::Class;
    }

    macro_rules! classes {
        ($($name:ident)+) => {
            $(
                #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
                pub struct $name;

                impl ConstClass for $name {
                    const CLASS: super::Class = super::Class::$name;
                }
            )+
        }
    }

    classes!(Universal Application Context Private);


    macro_rules! prefixes {
        ($($name:ident = $value:expr),+) => {
            $(
                #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
                pub struct $name;

                impl Prefix for $name {
                    const NAME: &'static str = $value;
                }
            )+
        }
    }

    prefixes! {
        ImplicitPrefix = "ASN.1#Implicit",
        ExplicitPrefix = "ASN.1#Explicit"
    }
}
