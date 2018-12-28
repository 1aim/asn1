use syn::{self, Error, spanned::Spanned};
use crate::{parse::Emit, attribute};

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
	pub fn parse<'b>(input: &'b syn::DeriveInput) -> Result<Container<'b>, syn::Error> {
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

				syn::Data::Enum(value) => {
					Data::Enum(value.variants.iter().filter_map(|v| Variant::parse(v).emit()).collect())
				}

				syn::Data::Union(_) => {
					return Err(Error::new(input.span(), "asn1: unions are not supported"));
				}
			}
		};

		Ok(this)
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
			syn::Data::Struct(value) =>
				&value.fields,

			_ =>
				unreachable!()
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
			syn::Fields::Named(value) =>
				(Style::Struct,
					value.named.iter().enumerate().filter_map(|(i, f)|
						Field::parse(i as u32, f).emit()).collect()),

			syn::Fields::Unnamed(value) =>
				(if value.unnamed.len() == 1 { Style::Newtype } else { Style::Tuple },
					value.unnamed.iter().enumerate().filter_map(|(i, f)|
						Field::parse(i as u32, f).emit()).collect()),

			syn::Fields::Unit =>
				(Style::Unit, vec![]),
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
		}
		else {
			syn::Member::Unnamed(syn::Index { index, span: input.span() })
		};

		Ok(Field {
			syn: input,
			member: member,
			ty: &input.ty,
			attributes: attribute::Field::parse(input)?,
		})
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
