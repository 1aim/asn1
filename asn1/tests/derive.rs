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

	let mut out = std::fs::File::create("/tmp/foo").unwrap();
	asn1::to_der(&mut out, Simple { a: 42, b: false }).unwrap();
}
