#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Class {
	Universal,
	Application,
	Context,
	Private,
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
		Tag { class, constructed, number }
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

pub const EOC: Tag = Tag { class: Class::Universal, constructed: false, number: 0x00 };
pub const BOOLEAN: Tag = Tag { class: Class::Universal, constructed: false, number: 0x01 };
pub const INTEGER: Tag = Tag { class: Class::Universal, constructed: false, number: 0x02 };
pub const BIT_STRING: Tag = Tag { class: Class::Universal, constructed: false, number: 0x03 };
pub const OCTET_STRING: Tag = Tag { class: Class::Universal, constructed: false, number: 0x04 };
pub const NULL: Tag = Tag { class: Class::Universal, constructed: false, number: 0x05 };
pub const OBJECT_ID: Tag = Tag { class: Class::Universal, constructed: false, number: 0x06 };
pub const SEQUENCE: Tag = Tag { class: Class::Universal, constructed: true, number: 0x10 };
pub const UTC_TIME: Tag = Tag { class: Class::Universal, constructed: false, number: 0x17 };
pub const GENERALIZED_TIME: Tag = Tag { class: Class::Universal, constructed: false, number: 0x18 };
