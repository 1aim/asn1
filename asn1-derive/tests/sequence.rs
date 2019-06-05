use asn1_der::{from_der, to_der};
use asn1_derive::Asn1;

#[test]
fn struct_of_bools() {
    #[derive(Asn1, Default, Debug, Clone, Copy, PartialEq)]
    struct Bools {
        a: bool,
        b: bool,
        c: bool,
    }

    let raw = &[
        0x30, // Sequence tag
        9,    // Length
        1, 1, 0, // A
        1, 1, 0, // B
        1, 1, 0, // C
    ][..];

    let default = Bools::default();

    let raw_default = to_der(default);

    assert_eq!(raw, &*raw_default);

    assert_eq!(default, from_der(&raw).unwrap());
}
