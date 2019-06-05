use crate::{attribute, parse::Emit};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{self, spanned::Spanned, Error};

/// A source data structure annotated with `#[derive(ASN1)]`, parsed into an
/// internal representation.
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: &'a syn::Ident,
    /// Attributes on the structure, parsed for Serde.
    pub attributes: attribute::Container,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
    /// Original input.
    pub syn: &'a syn::DeriveInput,
}

/// The fields of a struct or enum.
///
/// Analagous to `syn::Data`.
#[derive(Debug)]
pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

/// A variant of an enum.
#[derive(Debug)]
pub struct Variant<'a> {
    pub ident: &'a syn::Ident,
    pub attributes: attribute::Variant,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
    pub syn: &'a syn::Variant,
}

/// A field of a struct.
#[derive(Debug)]
pub struct Field<'a> {
    pub member: syn::Member,
    pub attributes: attribute::Field,
    pub ty: &'a syn::Type,
    pub syn: &'a syn::Field,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    Newtype,
    /// No fields.
    Unit,
}

impl<'a> Container<'a> {
    pub fn parse<'b>(input: &'b syn::DeriveInput) -> syn::Result<Container<'b>> {
        let this = Container {
            ident: &input.ident,
            syn: input,
            generics: &input.generics,
            attributes: attribute::Container::parse(&input)?,
            data: match &input.data {
                syn::Data::Struct(_) => {
                    let (style, fields) = Style::parse(input)?;
                    Data::Struct(style, fields)
                }

                syn::Data::Enum(value) => Data::Enum(
                    value
                        .variants
                        .iter()
                        .filter_map(|v| Variant::parse(v).emit())
                        .collect(),
                ),

                syn::Data::Union(_) => {
                    return Err(Error::new(input.span(), "asn1: unions are not supported"));
                }
            },
        };

        Ok(this)
    }

    /// Generate the decoder for DER.
    pub fn from_der(&self) -> syn::Result<TokenStream> {
        match self.data {
            Data::Enum(_) => self.enum_from_der(),
            Data::Struct(_, _) => self.struct_from_der(),
        }
    }

    /// Generate the encoder for DER.
    pub fn to_der(&self) -> syn::Result<TokenStream> {
        match self.data {
            Data::Enum(_) => self.enum_to_der(),
            Data::Struct(_, _) => self.struct_to_der(),
        }
    }

    fn enum_to_der(&self) -> syn::Result<TokenStream> {
        unimplemented!()
    }

    fn struct_to_der(&self) -> syn::Result<TokenStream> {
        let name = self.ident;
        let buffer_name = quote!(buffer);

        let (style, fields) = match &self.data {
            Data::Struct(style, fields) => (*style, fields),
            _ => unreachable!(),
        };

        if style == Style::Unit || style == Style::Tuple {
            panic!("Only named field structs are currently supported.")
        }

        let encode_fields: TokenStream = fields
            .iter()
            .map(|f| f.to_der(buffer_name.clone()))
            .collect();

        Ok(quote! {
            impl From<#name> for asn1_der::Value<Vec<u8>> {
                fn from(sequence: #name) -> Self {
                    let mut #buffer_name = Vec::new();

                    #encode_fields

                    asn1_der::Value::new(asn1_der::Tag::SEQUENCE, #buffer_name)
                }
            }
        })
    }

    fn enum_from_der(&self) -> syn::Result<TokenStream> {
        unimplemented!()
    }

    fn struct_from_der(&self) -> syn::Result<TokenStream> {
        let name = self.ident;
        let buffer_name = quote!(buffer);

        let (style, fields) = match &self.data {
            Data::Struct(style, fields) => (*style, fields),
            _ => unreachable!(),
        };

        if style == Style::Unit || style == Style::Tuple {
            panic!("Only named field structs are currently supported.")
        }

        let decode_fields: TokenStream = fields
            .iter()
            .map(|f| f.from_der(buffer_name.clone()))
            .collect();
        let field_names: TokenStream = fields.iter().map(Field::name).collect();

        Ok(quote! {
            impl<A: AsRef<[u8]>> std::convert::TryFrom<asn1_der::Value<A>> for #name {
                type Error = failure::Error;

                fn try_from(value: asn1_der::Value<A>) -> failure::Fallible<Self> {
                    let tag = value.tag;
                    failure::ensure!(tag == asn1_der::Tag::SEQUENCE, "{:?} is not tagged as a SEQUENCE", tag);
                    let mut #buffer_name = value.contents.as_ref();

                    #decode_fields

                    Ok(#name { #field_names })
                }
            }
        })
    }
}

/// Internal trait used to abstract over stuff that has fields, it's used by
/// [Style::parse].
trait Fields {
    fn fields(&self) -> &syn::Fields;
}

impl Fields for syn::DeriveInput {
    fn fields(&self) -> &syn::Fields {
        match &self.data {
            syn::Data::Struct(value) => &value.fields,

            _ => unreachable!(),
        }
    }
}

impl Fields for syn::Variant {
    fn fields(&self) -> &syn::Fields {
        &self.fields
    }
}

impl Style {
    /// Parse style and fields out of a struct-like item.
    fn parse<I: Fields + Spanned>(input: &I) -> Result<(Style, Vec<Field>), Error> {
        let (style, fields) = match input.fields() {
            syn::Fields::Named(value) => (
                Style::Struct,
                value
                    .named
                    .iter()
                    .enumerate()
                    .filter_map(|(i, f)| Field::parse(i as u32, f).emit())
                    .collect(),
            ),

            syn::Fields::Unnamed(value) => (
                if value.unnamed.len() == 1 {
                    Style::Newtype
                } else {
                    Style::Tuple
                },
                value
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(i, f)| Field::parse(i as u32, f).emit())
                    .collect(),
            ),

            syn::Fields::Unit => (Style::Unit, vec![]),
        };

        if style != Style::Unit && fields.is_empty() {
            return Err(Error::new(input.span(), "asn1: could not parse struct"));
        }

        Ok((style, fields))
    }
}

impl<'a> Field<'a> {
    pub fn parse<'b>(index: u32, input: &'b syn::Field) -> Result<Field<'b>, Error> {
        let member = if let Some(ident) = input.ident.as_ref() {
            syn::Member::Named(ident.clone())
        } else {
            syn::Member::Unnamed(syn::Index {
                index,
                span: input.span(),
            })
        };

        Ok(Field {
            syn: input,
            member: member,
            ty: &input.ty,
            attributes: attribute::Field::parse(input)?,
        })
    }

    pub fn name(&self) -> TokenStream {
        if let syn::Member::Named(name) = &self.member {
            quote! { #name , }
        } else {
            unreachable!();
        }
    }

    pub fn to_der(&self, buffer: TokenStream) -> TokenStream {
        if let syn::Member::Named(ref name) = self.member {
            quote! {
                let mut _tmp_buf = asn1_der::to_der(sequence.#name);
                #buffer.append(&mut _tmp_buf);
            }
        } else {
            unreachable!()
        }
    }

    pub fn from_der(&self, buffer: TokenStream) -> TokenStream {
        if let syn::Member::Named(ref name) = self.member {
            quote! {
                let (#buffer, #name) = asn1_der::from_der_partial(&#buffer)?;
            }
        } else {
            unreachable!()
        }
    }
}

impl<'a> Variant<'a> {
    pub fn parse<'b>(input: &'b syn::Variant) -> Result<Variant<'b>, Error> {
        let (style, fields) = Style::parse(input)?;

        Ok(Variant {
            ident: &input.ident,
            syn: input,
            style: style,
            fields: fields,
            attributes: attribute::Variant::parse(input)?,
        })
    }
}
