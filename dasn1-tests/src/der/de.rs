use dasn1::{
    AsnType,
    der::from_slice,
    types::*,
};

#[test]
fn bool() {
    let yes: bool = super::from_slice(&[0x1, 0x1, 0xFF][..]).unwrap();
    let no: bool = super::from_slice(&[0x1, 0x1, 0x0][..]).unwrap();

    assert!(yes);
    assert!(!no);
}

#[test]
fn choice() {
    #[derive(Clone, Debug, AsnType, PartialEq)]
    enum Foo {
        Ein,
        Zwei,
        Drei,
    }

    assert_eq!(Foo::Ein, from_slice(&[0x80, 0][..]).unwrap());
    assert_eq!(Foo::Zwei, from_slice(&[0x81, 0][..]).unwrap());
    assert_eq!(Foo::Drei, from_slice(&[0x82, 0][..]).unwrap());
}

#[test]
fn fixed_array_as_sequence() {
    let array = vec![8u8; 4];
    let raw = &[48, 4 * 3, 2, 1, 8, 2, 1, 8, 2, 1, 8, 2, 1, 8][..];
    assert_eq!(array, from_slice::<Vec<u8>>(&raw).unwrap());
}

#[test]
fn choice_newtype_variant() {
    #[derive(Clone, Debug, AsnType, PartialEq)]
    enum Foo {
        Bar(bool),
        Baz(OctetString),
    }

    let os = OctetString::from(vec![1, 2, 3, 4, 5]);

    assert_eq!(
        Foo::Bar(true),
        from_slice(&[0x80, 1, 0xff][..]).unwrap()
    );
    assert_eq!(
        Foo::Baz(os),
        from_slice(&[0x81, 5, 1, 2, 3, 4, 5][..]).unwrap()
    );
}

/*
   #[test]
   fn oid_from_bytes() {
   let oid = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();
   let from_raw =
   crate::from_slice(&[0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..]).unwrap();

   assert_eq!(oid, from_raw);
   }
   */
