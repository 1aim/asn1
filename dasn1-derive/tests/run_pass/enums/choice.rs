use dasn1::identifier::{AsnType, Class, Identifier};
use dasn1_derive::AsnType;

#[derive(AsnType)]
enum Choice {
    Foo(u8),
    Bar,
    Baz,
}

#[test]
fn uses_context_identifiers() {
    for (i, variant) in [Choice::Foo(0), Choice::Bar, Choice::Baz].into_iter().enumerate() {
        assert_eq!(variant.identifier(), Identifier::new(Class::Context, i as u32));
    }
}

