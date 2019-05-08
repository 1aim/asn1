use core::Class;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tag {
    pub class: Class,
    pub is_constructed: bool,
    pub tag: usize,
}

impl Tag {
    pub const EOC: Tag = Tag::new(Class::Universal, false, 0);
    pub const BOOL: Tag = Tag::new(Class::Universal, false, 1);
    pub const INTEGER: Tag = Tag::new(Class::Universal, false, 0x02);
    pub const BIT_STRING: Tag = Tag::new(Class::Universal, false, 0x03);
    pub const OCTET_STRING: Tag = Tag::new(Class::Universal, false, 0x04);
    pub const NULL: Tag = Tag::new(Class::Universal, false, 0x05);
    pub const OBJECT_IDENTIFIER: Tag = Tag::new(Class::Universal, false, 0x06);
    pub const SEQUENCE: Tag = Tag::new(Class::Universal, true, 0x10);
    pub const UTC_TIME: Tag = Tag::new(Class::Universal, false, 0x17);
    pub const GENERALIZED_TIME: Tag = Tag::new(Class::Universal, false, 0x18);

    pub const fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self {
            class,
            is_constructed,
            tag,
        }
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
