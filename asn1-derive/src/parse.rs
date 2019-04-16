use syn::{spanned::Spanned, Attribute, Error, Lit, LitInt, LitStr, Meta::List, NestedMeta};

macro_rules! try_parse {
    ($body:expr) => {
        if let Some(value) = $crate::parse::Emit::emit($body) {
            value
        } else {
            return $crate::proc_macro::TokenStream::new();
        }
    };
}

pub fn string(lit: &Lit) -> Result<&LitStr, Error> {
    match lit {
        Lit::Str(s) => Ok(s),

        _ => Err(Error::new(lit.span(), "asn1: expected string literal")),
    }
}

pub fn integer(lit: &Lit) -> Result<&LitInt, Error> {
    match lit {
        Lit::Int(i) => Ok(i),

        _ => Err(Error::new(lit.span(), "asn1: expected integer literal")),
    }
}

pub fn attributes(attr: &Attribute) -> Result<Vec<NestedMeta>, Error> {
    match attr.parse_meta()? {
        List(list) => {
            if list.ident == "asn1" {
                Ok(list.nested.into_pairs().map(|p| p.into_value()).collect())
            } else {
                Err(Error::new(
                    attr.span(),
                    "asn1: expected a list of attributes",
                ))
            }
        }

        _ => Err(Error::new(
            attr.span(),
            "asn1: expected a list of attributes",
        )),
    }
}

pub trait Emit<T> {
    fn emit(self) -> Option<T>;
}

impl<T> Emit<T> for Result<T, Error> {
    fn emit(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),

            Err(error) => {
                error.span().unstable().error(error.to_string()).emit();
                None
            }
        }
    }
}
