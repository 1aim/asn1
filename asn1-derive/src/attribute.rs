use syn::{self, Error, NestedMeta, Meta, spanned::Spanned, Data};
use crate::parse::{self, Emit};

#[derive(Copy, Clone)]
struct Attribute<T> {
	pub name:  &'static str,
	pub value: Option<T>,
}

#[derive(Debug)]
pub enum Default {
	None,
	Default,
	Path(syn::ExprPath)
}

#[derive(Debug)]
pub struct Container {
	pub name: String,
	pub implicit: Option<u64>,
	pub explicit: Option<u64>,
}

#[derive(Debug)]
pub struct Variant {
	pub name: String,
	pub implicit: Option<u64>,
	pub explicit: Option<u64>,
}

#[derive(Debug)]
pub struct Field {
	pub name: Option<String>,
	pub implicit: Option<u64>,
	pub explicit: Option<u64>,
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
		let is_struct = match item.data {
			Data::Struct(_) => true,
			_               => false,
		};

		let mut name = Attribute::none("rename");
		let mut implicit = Attribute::none("implicit");
		let mut explicit = Attribute::none("explicit");

		for meta_item in item.attrs.iter().filter_map(|a| parse::attributes(a).emit()).flatten() {
			match meta_item {
				// Parse `#[asn1(rename = "foo")]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "rename" => {
					if let Ok(s) = parse::string(&m.lit) {
						name.set(&m.ident, s.value()).emit();
					}
				}

				// Parse `#[asn1(implicit = 3)]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if is_struct && m.ident == "implicit" => {
					if let Ok(i) = parse::integer(&m.lit) {
						implicit.set(&m.ident, i.value()).emit();
					}
				}

				// Parse `#[asn1(explicit = 3)]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if is_struct && m.ident == "explicit" => {
					if let Ok(i) = parse::integer(&m.lit) {
						explicit.set(&m.ident, i.value()).emit();
					}
				}

				item => {
					item.span().unstable().warning(format!("asn1: unhandled item {:?}", item)).emit();
				}
			}
		}

		Ok(Self {
			name: name.value.unwrap_or_else(|| item.ident.to_string()),
			implicit: implicit.value,
			explicit: explicit.value,
		})
	}
}

impl Field {
	pub fn parse(item: &syn::Field) -> Result<Self, Error> {
		let mut name = Attribute::none("rename");
		let mut implicit = Attribute::none("implicit");
		let mut explicit = Attribute::none("explicit");

		for meta_item in item.attrs.iter().filter_map(|a| parse::attributes(a).emit()).flatten() {
			match meta_item {
				// Parse `#[asn1(rename = "foo")]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "rename" => {
					if let Ok(s) = parse::string(&m.lit) {
						name.set(&m.ident, s.value()).emit();
					}
				}

				// Parse `#[asn1(implicit = 3)]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "implicit" => {
					if let Ok(i) = parse::integer(&m.lit) {
						implicit.set(&m.ident, i.value()).emit();
					}
				}

				// Parse `#[asn1(explicit = 3)]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "explicit" => {
					if let Ok(i) = parse::integer(&m.lit) {
						explicit.set(&m.ident, i.value()).emit();
					}
				}

				item => {
					item.span().unstable().warning(format!("asn1: unhandled item {:?}", item)).emit();
				}
			}
		}

		Ok(Self {
			name: name.value.or_else(|| item.ident.clone().map(|i| i.to_string())),
			implicit: implicit.value,
			explicit: explicit.value,
		})
	}
}

impl Variant {
	pub fn parse(item: &syn::Variant) -> Result<Self, Error> {
		let mut name = Attribute::none("rename");
		let mut implicit = Attribute::none("implicit");
		let mut explicit = Attribute::none("explicit");

		for meta_item in item.attrs.iter().filter_map(|a| parse::attributes(a).emit()).flatten() {
			match meta_item {
				// Parse `#[asn1(rename = "foo")]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "rename" => {
					if let Ok(s) = parse::string(&m.lit) {
						name.set(&m.ident, s.value()).emit();
					}
				}

				// Parse `#[asn1(implicit = 3)]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "implicit" => {
					if let Ok(i) = parse::integer(&m.lit) {
						implicit.set(&m.ident, i.value()).emit();
					}
				}

				// Parse `#[asn1(explicit = 3)]`
				NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "explicit" => {
					if let Ok(i) = parse::integer(&m.lit) {
						explicit.set(&m.ident, i.value()).emit();
					}
				}

				item => {
					item.span().unstable().warning(format!("asn1: unhandled item {:?}", item)).emit();
				}
			}
		}

		Ok(Self {
			name: name.value.unwrap_or_else(|| item.ident.to_string()),
			implicit: implicit.value,
			explicit: explicit.value,
		})
	}
}
