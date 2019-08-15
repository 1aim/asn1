use syn::{DataEnum, Fields, Generics, Ident, Variant};

pub enum EnumKind {
    Choice,
    Enumerable,
}

impl EnumKind {
    pub fn from_variants<'a>(mut variants: impl Iterator<Item=&'a Variant>) -> Self {
        if variants.all(|v| v.fields == Fields::Unit) {
            EnumKind::Enumerable
        } else {
            EnumKind::Choice
        }
    }
}

pub struct Enum {
    pub kind: EnumKind,
    pub ident: Ident,
    pub generics: Generics,
    pub variants: Vec<Variant>,
}

impl Enum {
    pub fn new(ident: Ident, generics: Generics, data: DataEnum) -> Self {
        let variants = data.variants.into_iter().collect::<Vec<_>>();

        Self {
            kind: EnumKind::from_variants(variants.iter()),
            ident,
            generics,
            variants,
        }
    }
}
