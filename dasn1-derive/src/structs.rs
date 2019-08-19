use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Generics, Ident};

pub struct Struct {
    ident: Ident,
    generics: Generics,
    fields: Fields,
}

impl Struct {
    pub fn new(ident: Ident, generics: Generics, fields: Fields) -> Self {
        Self {
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
}
