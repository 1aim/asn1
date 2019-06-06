use asn1_der::{from_der, to_der};
use asn1_derive::Asn1;

#[test]
fn choice_of_structs() {
    #[derive(Asn1, Clone, Copy, Debug, PartialEq)]
    enum Foo {
        #[asn1(explicit = 0)]
        Bar(Bar),
        #[asn1(explicit = 1)]
        Baz(Baz),
    }

    #[derive(Asn1, Clone, Copy, Debug, PartialEq)]
    struct Bar {
        b: bool,
    }

    #[derive(Asn1, Clone, Copy, Debug, PartialEq)]
    struct Baz {
        eight: u8,
        sixteen: u16,
        thirty_two: u32,
        sixty_four: u64,
    }

    let a = Foo::Bar(Bar { b: true});
    let b = Foo::Baz(Baz { eight: 8, sixteen: 16, thirty_two: 32, sixty_four: 64});

    assert_eq!(a, from_der(&to_der(a)).unwrap());
    assert_eq!(b, from_der(&to_der(b)).unwrap());
}

