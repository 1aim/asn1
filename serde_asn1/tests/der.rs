extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_asn1 as asn1;

#[test]
fn serialize() {
	#[derive(Serialize)]
	struct Foo {
		a: i32,
		b: bool,
		c: (),
	}

	let mut vec = Vec::new();

	asn1::to_der(&mut vec, &Foo {
		a: 42,
		b: false,
		c: (),
	}).unwrap();

	println!("{:?}", vec);
}
