use dasn1_derive::AsnType;
use dasn1::per::PerEncodable;

#[test]
fn fixed_sequence() {
    #[derive(AsnType, Default)]
    #[asn(fixed)]
    struct Sequence {
        a: u8,
        b: u16,
        c: u8,
    }

    let seq = Sequence { a: 1, b: 2, c: 3 };
    let encoded = seq.encode();
    assert_eq!(32, encoded.len());
    assert_eq!(&[01, 00, 02, 03][..], &*encoded.to_bytes());
}

#[test]
fn extensible_sequence() {
    #[derive(AsnType, Default)]
    struct Sequence {
        a: u8,
        b: u16,
        c: u8,
    }

    assert_eq!(33, Sequence::default().encode().len());
}
