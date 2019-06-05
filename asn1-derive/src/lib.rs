// The `quote!` macro requires deep recursion.
#![recursion_limit = "512"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
mod parse;
mod ast;
mod attribute;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Asn1, attributes(asn1))]
pub fn derive_asn1(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let container = try_parse!(ast::Container::parse(&input));

    let to_der = try_parse!(container.to_der());
    let from_der = try_parse!(container.from_der());

    TokenStream::from(quote! {
        #from_der

        #to_der
    })
}
