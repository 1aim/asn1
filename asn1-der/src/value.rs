use core::Class;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tag {
    pub class: Class,
    pub is_constructed: bool,
    pub tag: usize,
}

impl Tag {
    pub fn new(class: Class, is_constructed: bool, tag: usize) -> Self {
        Self { class, is_constructed, tag }
    }

    pub fn set_tag(mut self, tag: usize) -> Self {
        self.tag = tag;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value<'a> {
    tag: Tag,
    pub(crate) contents: &'a [u8],
}

impl<'a> Value<'a> {
    pub fn new(tag: Tag, contents: &'a [u8]) -> Self {
        Self { tag, contents }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.tag.is_constructed || self.tag.tag != 1 {
            return None
        }

        Some(match self.contents[0] {
            0 => false,
            _ => true,
        })
    }
}

