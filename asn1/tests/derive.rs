#[macro_use]
extern crate asn1;

use asn1::ASN1;

#[test]
fn simple() {
	#[derive(ASN1)]
	struct Simple {
		a: u32,
		b: bool,
	}
}
