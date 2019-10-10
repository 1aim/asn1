extern crate proc_macro;

mod attributes;
mod enums;
mod structs;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Generics, Ident};

use enums::Enum;
use structs::Struct;

#[proc_macro_derive(AsnType, attributes(asn))]
pub fn my_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    // let metas = input.attrs.into_iter().filter_map(|a| a.parse_meta().ok());

    let generator = match input.data {
        Data::Struct(struct_data) => {
            Struct::new(name, generics, &input.attrs, struct_data.fields).into_trait_impl()
        }
        Data::Enum(enum_data) => Enum::new(name, generics, &input.attrs, enum_data).into_trait_impl(),
        _ => unimplemented!(),
    };

    proc_macro::TokenStream::from(generator)
}

trait AsnTypeGenerator: Sized {
    fn generics(&self) -> &Generics;
    fn name(&self) -> &Ident;
    fn generate_identifier_impl(&self) -> TokenStream;
    fn generate_tag_encoding_impl(&self) -> TokenStream {
        quote!()
    }

    fn generate_der_impl(&self) -> TokenStream {
        quote!()
    }

    fn generate_per_impl(&self) -> TokenStream {
        quote!()
    }

    fn into_trait_impl(self) -> proc_macro2::TokenStream {
        let name = self.name();
        let generics = self.generics();
        let identifier = self.generate_identifier_impl();
        let tag_encoding = self.generate_tag_encoding_impl();

        let der_encoding = if cfg!(feature = "der") {
            let der_impl = self.generate_der_impl();
            quote! {
                impl #generics dasn1::der::DerEncodable for #name #generics {
                    fn encode_value(&self) -> Vec<u8> {
                        #der_impl
                    }
                }
            }
        } else {
            quote!()
        };

        let per_encoding = if cfg!(feature = "per") {
            let per_impl = self.generate_per_impl();
            quote! {
                impl #generics dasn1::per::PerEncodable for #name #generics {
                    fn encode(&self) -> dasn1::per::ser::Buffer {
                        use dasn1::per::ConstrainedValue;

                        #per_impl
                    }
                }
            }
        } else {
            quote!()
        };

        let tag_encoding = if tag_encoding.is_empty() {
            tag_encoding
        } else {
            quote! {
                fn tag_encoding(&self) -> dasn1::identifier::TagEncoding {
                     #tag_encoding
                }
            }
        };

        quote! {
            impl #generics dasn1::AsnType for #name #generics {
                fn identifier(&self) -> dasn1::identifier::Identifier {
                    #identifier
                }

                #tag_encoding
            }

            #per_encoding
            #der_encoding
        }
    }
}
