// The `quote!` macro requires deep recursion.
#![recursion_limit = "512"]
#![feature(proc_macro_diagnostic)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

mod parse;
mod ast;
mod attribute;

use proc_macro2::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(ASN1, attributes(asn1))]
pub fn derive_asn1(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let container = ast::Container::parse(&input).unwrap();

	TokenStream::from(quote! {

	}).into()
}
