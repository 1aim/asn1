use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Fields, Generics, Ident, Variant};

use crate::attributes::EnumAttributes;

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
            EnumKind::Choice => self.create_pattern_match(format_ident!("self"), |i, _| {
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

    fn generate_tag_encoding_impl(&self) -> TokenStream {
        match self.kind {
            EnumKind::Enumerable => quote!(),
            EnumKind::Choice => quote!(), //self.create_pattern_match(format_ident!("self"), |_, fields| {})
        }
    }

    fn generate_per_impl(&self) -> TokenStream {
        match self.kind {
            EnumKind::Enumerable => self.generate_enumerable_per(),
            EnumKind::Choice => self.generate_choice_per(),
        }
    }
}

impl Enum {
    pub fn new(ident: Ident, generics: Generics, attrs: &[syn::Attribute], data: DataEnum) -> Self {
        let variants = data.variants.into_iter().collect::<Vec<_>>();

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
    pub fn create_pattern_match<F: Fn(usize, &[Ident]) -> TokenStream>(
        &self,
        match_ident: Ident,
        variant_arm_generator: F,
    ) -> TokenStream {
        let variants = self.variants.iter().enumerate().map(|(i, v)| {
            let enum_name = &self.ident;
            let variant_name = &v.ident;

            let (fields_pat, field_names) = if v.fields == Fields::Unit {
                (quote!(), Vec::new())
            } else {
                let fields = v
                    .fields
                    .iter()
                    .map(|f| f.ident.clone().unwrap_or(format_ident!("__f{}", i)))
                    .collect::<Vec<_>>();
                (quote!((#(#fields),*)), fields)
            };

            let arm = (variant_arm_generator)(i, &field_names);

            quote!(#enum_name::#variant_name #fields_pat => { #arm })
        });

        quote!(match #match_ident { #(#variants),*})
    }

    pub fn generate_choice_per(&self) -> TokenStream {
        let buf = format_ident!("buffer");
        let max_index = self.variants.iter().count();

        let encode_extensibility = if !self.attributes.container.fixed {
            quote!(#buf.push(false);)
        } else {
            quote!()
        };

        let encode_choice = self.create_pattern_match(format_ident!("self"), |index, fields| {
            let fields = fields.iter();
            quote! {
                #buf.push_field_list(#index.encode_with_constraint(0..#max_index));

                #(#buf.push_field_list(#fields.encode());)*

                #buf
            }
        });

        quote! {
            let mut #buf = dasn1::per::Buffer::new();
            #encode_extensibility

            #encode_choice
        }
    }

    pub fn generate_enumerable_per(&self) -> TokenStream {
        unimplemented!()
    }
}
