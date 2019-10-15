use syn::{Attribute, Lit, Meta, MetaList, NestedMeta};

/// Generic attributes common to all container types.
#[derive(Default)]
pub struct ContainerAttributes {
    /// If true don't encode extensibility bit.
    pub fixed: bool,
}

impl ContainerAttributes {
    pub fn from_syn(syn_attrs: &[Attribute]) -> Self {
        let mut attributes = Self::default();

        if let Some(list) = find_asn_attribute(syn_attrs) {
            for item in list.nested.iter().filter_map(|nm| match nm {
                NestedMeta::Meta(meta) => Some(meta),
                _ => None,
            }) {
                if item.path().is_ident("fixed") {
                    attributes.fixed = true;
                }
            }
        }

        attributes
    }
}

/// Enum specific attributes.
#[derive(Default)]
pub struct EnumAttributes {
    pub container: ContainerAttributes,
}

impl EnumAttributes {
    pub fn from_syn(attrs: &[Attribute]) -> Self {
        Self {
            container: ContainerAttributes::from_syn(attrs),
        }
    }
}

/// Struct specific attributes.
#[derive(Default)]
pub struct StructAttributes {
    pub container: ContainerAttributes,
}

impl StructAttributes {
    pub fn from_syn(attrs: &[Attribute]) -> Self {
        Self {
            container: ContainerAttributes::from_syn(attrs),
        }
    }
}

#[derive(Default)]
pub struct FieldAttributes {
    pub size: Option<Size>,
}

impl FieldAttributes {
    pub fn from_syn(syn_attrs: &[Attribute]) -> Self {
        let mut attributes = Self::default();
        let attribute_list = syn_attrs
            .iter()
            .filter_map(|a| a.parse_meta().ok())
            .filter(|m| m.path().is_ident("asn"))
            .next()
            .and_then(|m| match m {
                Meta::List(list) => Some(list.nested.into_iter().filter_map(|nm| match nm {
                    NestedMeta::Meta(m) => match m {
                        Meta::List(list) => Some(list),
                        _ => None,
                    },
                    _ => None,
                })),
                _ => None,
            });

        if let Some(list) = attribute_list {
            for item in list {
                if item.path.is_ident("size") {
                    attributes.size = Some(Size::from_syn(item));
                }
            }
        }

        attributes
    }
}

pub enum Size {
    Fixed(Lit),
    Range(Option<Lit>, Option<Lit>),
}

impl Size {
    pub fn from_syn(list: MetaList) -> Self {
        assert!(list.path.is_ident("size"));

        if list.nested.len() == 1 {
            match list.nested.first() {
                Some(NestedMeta::Lit(lit)) => Size::Fixed(lit.clone()),
                _ => panic!("Size attribute requires `min` and `max` or a int literal"),
            }
        } else {
            let mut start = None;
            let mut end = None;
            let lists = list.nested.iter().filter_map(|nm| match nm {
                NestedMeta::Meta(Meta::List(list)) => Some(list),
                _ => None,
            });

            for list in lists {
                let lit = list.nested.first().and_then(|nm| match nm {
                    NestedMeta::Lit(lit) => Some(lit.clone()),
                    _ => None,
                });

                if list.path.is_ident("min") {
                    start = lit;
                } else if list.path.is_ident("max") {
                    end = lit;
                }
            }

            Size::Range(start, end)
        }
    }
}

fn find_asn_attribute(attrs: &[Attribute]) -> Option<MetaList> {
    attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .find(|m| m.path().is_ident("asn"))
        .and_then(|meta| match meta {
            Meta::List(list) => Some(list),
            _ => None,
        })
}
