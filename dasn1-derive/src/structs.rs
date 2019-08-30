mod attributes;

pub use self::attributes::{FieldAttributes, StructAttributes, Size};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Generics, Ident, Type};

pub struct Struct {
    ident: Ident,
    generics: Generics,
    fields: Fields,
    attributes: StructAttributes
}

impl Struct {
    pub fn new(ident: Ident, generics: Generics, attrs: &[syn::Attribute], fields: Fields) -> Self {
        Self {
            attributes: StructAttributes::from_syn(attrs),
            ident,
            generics,
            fields,
        }
    }
}

impl super::AsnTypeGenerator for Struct {
    fn name(&self) -> &Ident {
        &self.ident
    }

    fn generics(&self) -> &Generics {
        &self.generics
    }

    fn generate_identifier_impl(&self) -> TokenStream {
        match self.fields {
            Fields::Unit => quote!(dasn1::identifier::Identifier::NULL),
            _ => quote!(dasn1::identifier::Identifier::SEQUENCE),
        }
    }

    fn generate_per_impl(&self) -> TokenStream {
        let buf = format_ident!("buffer");

        let optional_fields_iter = self.fields.iter()
            // Enumerate first to get field order to be able to correctly access
            // tuple struct fields.
            .enumerate()
            .filter(|(_, f)| match f.ty {
                Type::Path(ref type_path) => {
                    type_path
                        .path
                        .segments
                        .first()
                        .map(|s| s.ident == "Option")
                        .unwrap_or(false)
                },
                _ => false,
            })
            .map(|(i, f)| f.ident.clone().unwrap_or_else(|| format_ident!("{}", i)))
            .map(|ident| quote!(#buf.push(self.#ident.is_some());));


        let fields_iter = self.fields.iter()
            .enumerate()
            .map(|(i, f)| {
                let ident = f.ident.clone().unwrap_or_else(|| format_ident!("{}", i));
                let attributes = FieldAttributes::from_syn(&f.attrs);

                if let Some(size) = attributes.size {
                    match size {
                        Size::Fixed(_) => unimplemented!(),
                        Size::Range(start, end) => {
                            quote!(self.#ident.encode_with_constraint(#start..=#end))
                        }
                    }
                } else {
                    quote!(self.#ident.encode())
                }
            });

        let encode_extensibility = if !self.attributes.fixed {
            quote!(#buf.push(false);)
        } else {
            quote!()
        };

        quote! {
            let mut buffer = dasn1::per::Buffer::new();
            #encode_extensibility

            #(#optional_fields_iter)*

            #(buffer.push_field_list(#fields_iter);)*

            buffer
        }
    }
}
