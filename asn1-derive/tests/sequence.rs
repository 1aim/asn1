use asn1_derive::Asn1;
use asn1_der::{from_der, to_der};

#[test]
fn struct_of_bools() {
    #[derive(Asn1, Default, Debug, Clone, Copy, PartialEq)]
    struct Bools {
        a: bool,
        b: bool,
        c: bool,
    }

    let raw = &[
        // Sequence tag
        0x30,
        // Length
        9,
        // A
        1, 1, 0,
        // B
        1, 1, 0,
        // C
        1, 1, 0,
    ][..];

    let default = Bools::default();

    let raw_default = to_der(default);

    assert_eq!(raw, &*raw_default);

    assert_eq!(default, from_der(&raw).unwrap());
}
