use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Generics, Ident, Lit, Meta, NestedMeta, Type};

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

    fn generate_per_impl(&self) -> TokenStream {
        let buf = format_ident!("buffer");
        let num_of_optional_fields = self.fields.iter()
            .filter_map(|f| match f.ty {
                Type::Path(ref type_path) => type_path.path.get_ident(),
                _ => None,
            })
            .filter(|ident| ident.to_string().contains("Option<"))
            .count();

        let optional_fields_iter = self.fields.iter()
            // Enumerate first to get field order to be able to correctly access
            // tuple struct fields.
            .enumerate()
            .filter(|(_, f)| match f.ty {
                Type::Path(ref type_path) => type_path.path.segments.first().map(|s| s.ident == "Option").unwrap_or(false),
                _ => false,
            })
            .map(|(i, f)| f.ident.clone().unwrap_or_else(|| format_ident!("{}", i)))
            .map(|ident| quote!(#buf.push(self.#ident.is_some());));


        let fields_iter = self.fields.iter()
            .enumerate()
            .map(|(i, f)| {
                let ident = f.ident.clone().unwrap_or_else(|| format_ident!("{}", i));

                let size_attribute = {
                    let mut asn_attributes = f.attrs.iter()
                        .filter_map(|a| a.parse_meta().ok())
                        .filter(|m| m.path().is_ident("asn"));

                    asn_attributes.next().and_then(|m| match m {
                        Meta::List(list) => {
                            let mut start = None;
                            let mut end = None;

                            let meta_items = list.nested.iter().filter_map(|nm| match nm {
                                NestedMeta::Meta(Meta::List(list)) => Some(list),
                                _ => None,
                            });

                            for item in meta_items {
                                if item.path.is_ident("start") {
                                    start = Some(item.nested.iter().next().and_then(|nm| match nm {
                                        NestedMeta::Lit(Lit::Int(int_lit)) => Some(int_lit.clone()),
                                        _ => None,
                                    }));
                                } else if item.path.is_ident("end") {
                                    end = Some(item.nested.iter().next().and_then(|nm| match nm {
                                        NestedMeta::Lit(Lit::Int(int_lit)) => Some(int_lit.clone()),
                                        _ => None,
                                    }));
                                }
                            }

                            Some((start, end))
                        },
                        _ => None,
                    })
                };

                if let Some((start, end)) = size_attribute {
                    quote!(dasn1::per::ser::number::encode_constrained_whole_number(self.#ident, #start..=#end))
                } else {
                    quote!(self.#ident.encode())
                }
            });

        quote! {
            let mut buffer = dasn1::per::Buffer::new();
            // Encode extensible bit.
            buffer.push(false);

            #(#optional_fields_iter)*

            #(buffer.append(&mut #fields_iter);)*

            buffer
        }
    }
}
