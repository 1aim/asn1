extern crate asn1;
#[macro_use]
extern crate asn1_derive;

#[test]
fn simple() {
	#[derive(ASN1)]
	#[asn1(rename = "Complex")]
	struct Simple {
		#[asn1(rename = "Complex")]
		a: u32,
		b: bool,
	}

	let mut out = Vec::new();
	asn1::to_der(&mut out, Simple { a: 42, b: false }).unwrap();
	assert_eq!(out, &[0x30, 0x06, 0x02, 0x01, 0x2a, 0x01, 0x01, 0x00]);
}
