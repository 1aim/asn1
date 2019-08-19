use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{DataEnum, Fields, Generics, Ident, Variant};

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
            EnumKind::Choice => self.create_pattern_match(format_ident!("self"), |_, fields| {
                if fields.is_empty() {
                    quote!()
                } else {
                    let field = &fields[0];
                    quote!(#field.tag_encoding())
                }
            })
        }
    }
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

    /// Generates a match expression for `match_ident`, and calls
    /// `variant_arm_generator` for each arm of the match expression, providing
    /// the function with the index of each variant as well as list of `Ident`s
    /// for that variant's fields. An empty list is equivalvent to a
    /// unit variant.
    pub fn create_pattern_match(
        &self,
        match_ident: Ident,
        variant_arm_generator: fn(usize, &[Ident]) -> TokenStream,
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

            quote!(#enum_name::#variant_name #fields_pat => #arm)
        });

        quote!(match #match_ident { #(#variants),*})
    }
}
