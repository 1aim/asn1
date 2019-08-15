extern crate proc_macro;

mod enums;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Path};

use enums::EnumKind;

#[proc_macro_derive(AsnType)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    // let metas = input.attrs.into_iter().filter_map(|a| a.parse_meta().ok());

    let identifier = match input.data {
        Data::Struct(struct_data) => match struct_data.fields {
            Fields::Unit => quote!(dasn1::identifier::Identifier::NULL),
            _ => quote!(dasn1::identifier::Identifier::SEQUENCE),
        },
        Data::Enum(enum_data) => match EnumKind::from_variants(enum_data.variants.iter()) {
            EnumKind::Enumerable => quote!(dasn1::identifier::Identifier::ENUMERATED),
            EnumKind::Choice => {
                let variants =
                    enum_data.variants.iter().enumerate().map(
                        |(i, v)| quote!(#name::#v => Identifier::new(Class::Context, #i as u32)),
                    );

                quote! {
                    match self {
                        #(#variants),*
                    }
                }
            }
        },
        _ => unimplemented!(),
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #generics dasn1::identifier::AsnType for #name #generics {
            fn identifier(&self) -> dasn1::identifier::Identifier {
                #identifier
            }
        }
    };

    TokenStream::from(expanded)
}
