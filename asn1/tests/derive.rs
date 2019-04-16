extern crate asn1;
use asn1::ASN1;

#[test]
fn simple() {
    #[derive(ASN1)]
    struct Simple {
        a: u32,
        b: bool,
    }

    #[cfg(feature = "der")]
    {
        let mut out = Vec::new();
        asn1::to_der(&mut out, Simple { a: 42, b: false }).unwrap();
        assert_eq!(out, &[0x30, 0x06, 0x02, 0x01, 0x2a, 0x01, 0x01, 0x00]);
    }
}

#[test]
fn choice() {
    #[derive(ASN1)]
    enum Choice {
        Foo(u32),
        Bar(bool),
    }

    #[cfg(feature = "der")]
    {
        let mut out = Vec::new();
        asn1::to_der(&mut out, Choice::Foo(42)).unwrap();
        println!("{:?}", out);

        let mut out = Vec::new();
        asn1::to_der(&mut out, Choice::Bar(false)).unwrap();
        println!("{:?}", out);
    }
}
