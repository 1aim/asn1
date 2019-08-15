use dasn1::identifier::{AsnType, Identifier};
use dasn1_derive::AsnType;

#[derive(AsnType)]
enum Enum {
    Foo,
    Bar,
    Baz,
}

#[test]
fn uses_enumerable_identifier() {
    for variant in &[Enum::Foo, Enum::Bar, Enum::Baz] {
        assert_eq!(variant.identifier(), Identifier::ENUMERATED);
    }
}
