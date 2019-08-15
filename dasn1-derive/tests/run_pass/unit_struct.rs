use dasn1::identifier::{AsnType, Identifier};
use dasn1_derive::AsnType;

#[derive(AsnType)]
struct Foo;


#[test]
fn uses_sequence_identifier() {
    assert_eq!(Foo.identifier(), Identifier::NULL);
}
