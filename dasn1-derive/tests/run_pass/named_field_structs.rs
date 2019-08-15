use dasn1::identifier::*;
use dasn1_derive::*;

#[derive(AsnType, Default)]
struct Sequence {
    a: u8,
    b: i8,
    c: String,
}

#[test]
fn named_struct_identifier_is_sequence() {
    let sequence = Sequence::default();
    assert_eq!(sequence.identifier(), Identifier::SEQUENCE);
}
