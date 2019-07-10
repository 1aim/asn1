use failure::{ensure, Fallible};
use serde::{Deserialize, Serialize};

/// A representation of the `OBJECT IDENTIFIER` ASN.1 data type. Use an
/// `OBJECT IDENTIFIER` when a compact numerical identification of a node of the
/// OID tree is needed in binary encodings.
/// # Example
/// ```asn1
/// -- NumericString ASN.1 type (see X.680 41.3) --
/// numericString OBJECT IDENTIFIER ::=
/// { joint-iso-itu-t asn1(1) specification(0) characterStrings(1) numericString(0) }
///
/// -- PrintableString ASN.1 type (see X.680 41.5) --
/// printableString OBJECT IDENTIFIER ::=
/// { joint-iso-itu-t asn1(1) specification(0) characterStrings(1) printableString(1) }
///
/// -- BER encoding of a single ASN.1 type --
/// ber OBJECT IDENTIFIER ::=
/// { joint-iso-itu-t asn1(1) basic-encoding(1) }
///
/// -- CER encoding of a single ASN.1 type --
/// cer OBJECT IDENTIFIER ::=
/// { joint-iso-itu-t asn1(1) ber-derived(2) canonical-encoding(0) }
///
/// -- DER encoding of a single ASN.1 type --
/// der OBJECT IDENTIFIER ::=
/// { joint-iso-itu-t asn1(1) ber-derived(2) distinguished-encoding(1) }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename = "ASN.1#ObjectIdentifier")]
pub struct ObjectIdentifier<T: AsRef<[u128]>>(pub T);

impl<A: AsRef<[u128]>> ObjectIdentifier<A> {
    pub fn new(inner: A) -> Fallible<Self> {
        ensure!(
            inner.as_ref().len() >= 2,
            "ObjectIdentifier requires at least two components."
        );

        Ok(Self(inner))
    }
}

impl<T: AsRef<[u128]>> AsRef<[u128]> for ObjectIdentifier<T> {
    fn as_ref(&self) -> &[u128] {
        self.0.as_ref()
    }
}
