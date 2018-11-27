// The `quote!` macro requires deep recursion.
#![recursion_limit = "512"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(ASN1, attributes(asn1))]
pub fn derive_asn1(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	(quote! { }).into()
}
