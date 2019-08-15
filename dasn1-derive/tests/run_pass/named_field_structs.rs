use dasn1::identifier::{AsnType, Identifier};
use dasn1_derive::AsnType;

#[derive(AsnType, Default)]
struct Sequence {
    a: u8,
    b: i8,
    c: String,
}

#[test]
fn uses_sequence_identifier() {
    let sequence = Sequence::default();
    assert_eq!(sequence.identifier(), Identifier::SEQUENCE);
}
