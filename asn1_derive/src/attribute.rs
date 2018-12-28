use syn::{self, Error, NestedMeta, Meta, spanned::Spanned};
use crate::parse::{self, Emit};

#[derive(Copy, Clone)]
struct Attribute<T> {
	name:  &'static str,
	value: Option<T>,
}

#[derive(Debug)]
pub enum Default {
	None,
	Default,
	Path(syn::ExprPath)
}

#[derive(Debug)]
pub struct Container {
	name: String,
}

#[derive(Debug)]
pub struct Variant {
	name: String,
}

#[derive(Debug)]
pub struct Field {
	name: String,
}

impl<T> Attribute<T> {
	pub fn none(name: &'static str) -> Self {
		Attribute { name, value: None }
	}

	pub fn set<Span: Spanned>(&mut self, span: Span, value: T) -> Result<(), Error> {
		if self.value.is_none() {
			self.value = Some(value);
			Ok(())
		}
		else {
			Err(Error::new(span.span(), format!("asn1: duplicate attribute: {}", self.name)))
		}
	}
}

impl Container {
	pub fn parse(item: &syn::DeriveInput) -> Result<Self, Error> {
		let mut name = Attribute::none("rename");

		for meta_item in item.attrs.iter().filter_map(|a| parse::attributes(a).emit()).flatten() {
			match meta_item {
				// Parse `#[asn1(rename = "foo")]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "rename" => {
					if let Ok(s) = parse::string(&m.lit) {
						name.set(&m.ident, s.value()).emit();
					}
				}

				item => {
					item.span().unstable().warning(format!("asn1: unhandled item {:?}", item)).emit();
				}
			}
		}

		Ok(Self {
			name: name.value.unwrap(),
		})
	}
}

impl Field {
	pub fn parse(item: &syn::Field) -> Result<Self, Error> {
		let mut name = Attribute::none("rename");

		for meta_item in item.attrs.iter().filter_map(|a| parse::attributes(a).emit()).flatten() {
			match meta_item {
				// Parse `#[asn1(rename = "foo")]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "rename" => {
					if let Ok(s) = parse::string(&m.lit) {
						name.set(&m.ident, s.value()).emit();
					}
				}

				item => {
					item.span().unstable().warning(format!("asn1: unhandled item {:?}", item)).emit();
				}
			}
		}

		Ok(Self {
			name: name.value.unwrap(),
		})
	}
}

impl Variant {
	pub fn parse(item: &syn::Variant) -> Result<Self, Error> {
		let mut name = Attribute::none("rename");

		for meta_item in item.attrs.iter().filter_map(|a| parse::attributes(a).emit()).flatten() {
			match meta_item {
				// Parse `#[asn1(rename = "foo")]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "rename" => {
					if let Ok(s) = parse::string(&m.lit) {
						name.set(&m.ident, s.value()).emit();
					}
				}

				item => {
					item.span().unstable().warning(format!("asn1: unhandled item {:?}", item)).emit();
				}
			}
		}

		Ok(Self {
			name: name.value.unwrap(),
		})
	}
}
