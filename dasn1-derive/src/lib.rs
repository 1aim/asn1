extern crate proc_macro;

mod enums;
mod structs;

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Generics, Ident};

use enums::Enum;
use structs::Struct;

#[proc_macro_derive(AsnType)]
pub fn my_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    // let metas = input.attrs.into_iter().filter_map(|a| a.parse_meta().ok());

    let generator = match input.data {
        Data::Struct(struct_data) => Struct::new(name, generics, struct_data.fields).into_trait_impl(),
        Data::Enum(enum_data) => Enum::new(name, generics, enum_data).into_trait_impl(),
        _ => unimplemented!(),
    };

    proc_macro::TokenStream::from(generator)
}

trait AsnTypeGenerator: Sized {
    fn generics(&self) -> &Generics;
    fn name(&self) -> &Ident;
    fn generate_identifier_impl(&self) -> TokenStream;

    fn into_trait_impl(self) -> proc_macro2::TokenStream {
        let name = self.name();
        let generics = self.generics();
        let identifier = self.generate_identifier_impl();

        quote! {
            impl #generics dasn1::identifier::AsnType for #name #generics {
                fn identifier(&self) -> dasn1::identifier::Identifier {
                    #identifier
                }
            }
        }
    }
}
