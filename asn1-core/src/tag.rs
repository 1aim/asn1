use std::convert::TryFrom;

use failure::Fallible;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Class {
    Universal = 0,
    Application,
    Context,
    Private,
}

impl TryFrom<u8> for Class {
    type Error = failure::Error;

    fn try_from(byte: u8) -> Fallible<Self> {
        match byte {
            0 => Ok(Class::Universal),
            1 => Ok(Class::Application),
            2 => Ok(Class::Context),
            3 => Ok(Class::Private),
            _ => Err(failure::err_msg("asn1: Invalid kind of Class.")),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Tag {
    /// The tag class.
    pub class: Class,

    /// Whether the type is a construct or not.
    pub constructed: bool,

    /// TODO(meh): This is actually supposed to be a BigInt.
    pub number: u8,
}

impl Tag {
    pub fn new(class: Class, constructed: bool, number: u8) -> Tag {
        Tag {
            class,
            constructed,
            number,
        }
    }

    pub fn universal(number: u8) -> Tag {
        Self::new(Class::Universal, false, number)
    }

    pub fn application(number: u8) -> Tag {
        Self::new(Class::Application, false, number)
    }

    pub fn context(number: u8) -> Tag {
        Self::new(Class::Context, false, number)
    }

    pub fn private(number: u8) -> Tag {
        Self::new(Class::Private, false, number)
    }

    pub fn constructed(self, value: bool) -> Tag {
        Self::new(self.class, value, self.number)
    }
}

/// TODO(meh): This is wrong, needs to support `BigInt`.
///
/// Low-tag-number form. One octet. Bits 8 and 7 specify the class (see Table
/// 2), bit 6 has value "0," indicating that the encoding is primitive, and
/// bits 5-1 give the tag number.
///
/// High-tag-number form. Two or more octets. First octet is as in
/// low-tag-number form, except that bits 5-1 all have value "1." Second and
/// following octets give the tag number, base 128, most significant digit
/// first, with as few digits as possible, and with the bit 8 of each octet
/// except the last set to "1."
impl From<u8> for Tag {
    fn from(value: u8) -> Tag {
        Tag::new(Class::Universal, false, value)
    }
}
