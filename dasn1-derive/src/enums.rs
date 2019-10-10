use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Fields, Generics, Ident};

use crate::attributes::EnumAttributes;

#[derive(Clone, Copy, Debug)]
pub enum EnumKind {
    Choice,
    Enumerable,
}

impl EnumKind {
    pub fn from_variants<'a>(mut variants: impl Iterator<Item = &'a Variant>) -> Self {
        if variants.all(|v| v.fields == Fields::Unit) {
            EnumKind::Enumerable
        } else {
            EnumKind::Choice
        }
    }
}

pub struct Enum {
    pub attributes: EnumAttributes,
    pub kind: EnumKind,
    pub ident: Ident,
    pub generics: Generics,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantKind {
    Unit,
    Tuple,
    Struct
}

impl VariantKind {
    fn is_struct(self) -> bool {
        match self {
            Self::Struct => true,
            _ => false,
        }
    }
}

impl From<&Fields> for VariantKind {
    fn from(fields: &Fields) -> Self {
        match fields {
            Fields::Named(_) => Self::Struct,
            Fields::Unnamed(_) => Self::Tuple,
            Fields::Unit => Self::Unit,
        }
    }
}

pub struct Variant {
    attrs: Vec<syn::Attribute>,
    ident: Ident,
    fields: Fields,
    kind: VariantKind,
}

impl From<syn::Variant> for Variant {
    fn from(var: syn::Variant) -> Self {
        Self {
            attrs: var.attrs,
            ident: var.ident,
            kind: VariantKind::from(&var.fields),
            fields: var.fields
        }
    }
}

impl super::AsnTypeGenerator for Enum {
    fn name(&self) -> &Ident {
        &self.ident
    }

    fn generics(&self) -> &Generics {
        &self.generics
    }

    fn generate_identifier_impl(&self) -> TokenStream {
        match self.kind {
            EnumKind::Enumerable => quote!(dasn1::identifier::Identifier::ENUMERATED),
            EnumKind::Choice => self.create_pattern_match(|i, _, _| {
                let i = i as u32;

                quote!(
                    dasn1::identifier::Identifier::new(
                        dasn1::identifier::Class::Context,
                        #i
                    )
                )
            })
        }
    }

    fn generate_per_impl(&self) -> TokenStream {
        let buf = format_ident!("buffer");

        let encode_extensibility = if !self.attributes.container.fixed {
            quote!(#buf.push(false);)
        } else {
            quote!()
        };

        let encode_enum = match self.kind {
            EnumKind::Enumerable => self.generate_enumerable_per(&buf),
            EnumKind::Choice => self.generate_choice_per(&buf),
        };

        quote! {
            let mut #buf = dasn1::per::Buffer::new();
            #encode_extensibility

            #encode_enum
        }
    }

    fn generate_der_impl(&self) -> TokenStream {
        let buf = format_ident!("buffer");

        let encode_enum = match self.kind {
            EnumKind::Enumerable => self.generate_enumerable_der(&buf),
            EnumKind::Choice => self.generate_choice_der(&buf),
        };

        quote! {
            let mut #buf = Vec::new();

            #encode_enum
        }
    }
}

impl Enum {
    pub fn new(ident: Ident, generics: Generics, attrs: &[syn::Attribute], data: DataEnum) -> Self {
        let variants = data.variants.into_iter().map(Variant::from).collect::<Vec<_>>();

        Self {
            attributes: EnumAttributes::from_syn(attrs),
            kind: EnumKind::from_variants(variants.iter()),
            ident,
            generics,
            variants,
        }
    }

    /// Generates a match expression for `match_ident`, and calls
    /// `variant_arm_generator` for each arm of the match expression, providing
    /// the function with the index of each variant as well as list of `Ident`s
    /// for that variant's fields. An empty list is equivalvent to a
    /// unit variant.
    pub fn create_pattern_match<F: Fn(usize, &Variant, &[Ident]) -> TokenStream>(
        &self,
        variant_arm_generator: F,
    ) -> TokenStream {
        let variants = self.variants.iter().enumerate().map(|(i, v)| {
            let enum_name = &self.ident;
            let variant_name = &v.ident;

            let fields = v
                .fields
                .iter()
                .map(|f| f.ident.clone().unwrap_or(format_ident!("__f{}", i)))
                .collect::<Vec<_>>();

            let (fields_pat, field_names) = match v.kind {
                VariantKind::Tuple => (quote!((#(#fields),*)), fields),
                VariantKind::Struct => (quote!({ #(#fields),* }), fields),
                VariantKind::Unit => (quote!(), Vec::new()),
            };

            let arm = (variant_arm_generator)(i, &v, &field_names);

            quote!(#enum_name::#variant_name #fields_pat => { #arm })
        });

        quote!(match self { #(#variants),*})
    }

    pub fn generate_choice_per(&self, buf: &Ident) -> TokenStream {
        let max_index = self.variants.iter().count();

        let encode_choice = self.create_pattern_match(|index, _, fields| {
            let fields = fields.iter();
            quote! {
                #buf.push_field_list(#index.encode_with_constraint(0..#max_index));

                #(#buf.push_field_list(#fields.encode());)*

                #buf
            }
        });

        quote! {
            #encode_choice
        }
    }

    pub fn generate_enumerable_per(&self, buf: &Ident) -> TokenStream {
        quote!(#buf)
    }

    pub fn generate_enumerable_der(&self, buf: &Ident) -> TokenStream {
        self.create_pattern_match(|index, _, _| {
            quote! {
                #buf.push(#index as u8);

                #buf
            }
        })
    }

    pub fn generate_choice_der(&self, buf: &Ident) -> TokenStream {
        self.create_pattern_match(|_, variant, fields| {
            let fields = fields.iter();

            // If there's only a multiple fields or it's a struct enum we treat
            // it the same as `Foo.b` below other wise we treat like `Foo.a`.
            // ```
            // CHOICE Foo {
            //     a Field,
            //     b SEQUENCE {
            //         c Field,
            //         d Field
            //     }
            // }
            // ```
            let field_code = if fields.len() == 1 && !variant.kind.is_struct() {
                quote!(#(#buf.append(&mut #fields.encode_value());)*)
            } else {
                let fields = fields.enumerate().map(|(i, field)| {
                    let i = i as u32;
                    quote!(
                        #buf.append(&mut
                            #field.encode_implicit(
                                dasn1::identifier::Identifier::new(
                                    dasn1::identifier::Class::Context,
                                    #i
                                )
                            )
                        );
                    )
                });

                quote!(#(#fields)*)
            };

            quote! {
                #field_code

                #buf
            }
        })
    }
}
