use dasn1::identifier::*;
use dasn1_derive::*;

#[derive(AsnType)]
enum Enum {
    Foo,
    Bar,
    Baz,
}

#[test]
fn unit_only_enum_identifier_is_enumerated() {
    for variant in &[Enum::Foo, Enum::Bar, Enum::Baz] {
        assert_eq!(variant.identifier(), Identifier::ENUMERATED);
    }
}
