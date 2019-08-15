use dasn1::identifier::*;
use dasn1_derive::*;

#[derive(AsnType)]
struct Null;


#[test]
fn unit_struct_identifier_is_null() {
    assert_eq!(Null.identifier(), Identifier::NULL);
}
