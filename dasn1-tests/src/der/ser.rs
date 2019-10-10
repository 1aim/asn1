use dasn1::{AsnType, der::to_vec, identifier::constant::*, types::*};

#[test]
fn bool() {
    assert_eq!(&[1, 1, 255][..], &*to_vec(&true).unwrap());
    assert_eq!(&[1, 1, 0][..], &*to_vec(&false).unwrap());
}

#[test]
fn universal_string() {
    assert_eq!(
        &[28, 5, 0x4A, 0x6F, 0x6E, 0x65, 0x73][..],
        &*to_vec(&"Jones").unwrap()
    );
}

#[test]
fn fixed_array_as_sequence() {
    let array = [8u8; 4];
    assert_eq!(
        &[48, 4 * 3, 2, 1, 8, 2, 1, 8, 2, 1, 8, 2, 1, 8][..],
        &*to_vec(&array).unwrap()
    );
}

#[test]
fn encode_long_sequence() {
    let vec = vec![5; 0xffff];
    let preamble = vec![0x30u8, 0x83, 0x2, 0xFF, 0xFD];
    assert_eq!(&*preamble, &to_vec(&vec).unwrap()[..preamble.len()]);
}

#[test]
fn sequence_with_option() {
    #[derive(Clone, Debug, AsnType, PartialEq)]
    struct Foo {
        a: u8,
        b: Option<u8>,
    }

    let some = Foo { a: 1, b: Some(2) };
    let none = Foo { a: 1, b: None };

    assert_eq!(
        &[0x30, 3 * 2, 0x80, 0x1, 0x1, 0x81, 0x1, 0x2][..],
        &*to_vec(&some).unwrap()
    );
    assert_eq!(
        &[0x30, 0x3, 0x80, 0x1, 0x1][..],
        &*to_vec(&none).unwrap()
    );
}


#[test]
fn enumerated() {
    #[derive(Clone, Debug, AsnType, PartialEq)]
    enum Numbers {
        Ein,
        Zwei,
        Drei,
    }

    let ein = Numbers::Ein;
    let zwei = Numbers::Zwei;
    let drei = Numbers::Drei;

    assert_eq!(&[0xA, 1, 0][..], &*to_vec(&ein).unwrap());
    assert_eq!(&[0xA, 1, 1][..], &*to_vec(&zwei).unwrap());
    assert_eq!(&[0xA, 1, 2][..], &*to_vec(&drei).unwrap());
}

#[test]
fn choice() {
    #[derive(Clone, Debug, AsnType, PartialEq)]
    enum Numbers {
        Ein(u8),
        Zwei(u32),
        Drei(u64),
    }

    let ein = Numbers::Ein(0);
    let zwei = Numbers::Zwei(1);
    let drei = Numbers::Drei(2);

    assert_eq!(&[0x80, 1, 0][..], &*to_vec(&ein).unwrap());
    assert_eq!(&[0x81, 1, 1][..], &*to_vec(&zwei).unwrap());
    assert_eq!(&[0x82, 1, 2][..], &*to_vec(&drei).unwrap());
}


#[test]
fn choice_newtype_variant() {
    #[derive(Clone, Debug, AsnType, PartialEq)]
    enum Foo {
        Bar(bool),
        Baz(OctetString),
        Blah(Blah),
        Lah {
            os: OctetString,
        }
    }

    #[derive(Clone, Debug, AsnType, PartialEq)]
    struct Blah {
        data: OctetString,
    }

    let os = OctetString::from(vec![1, 2, 3, 4, 5]);
    let bar = Foo::Bar(true);
    let baz = Foo::Baz(os.clone());
    let blah = Foo::Blah(Blah { data: os.clone() });
    let lah = Foo::Lah { os };

    assert_eq!(&[0x80, 1, 0xff][..], &*to_vec(&bar).unwrap());
    assert_eq!(&[0x81, 5, 1, 2, 3, 4, 5][..], &*to_vec(&baz).unwrap());
    assert_eq!(&[0x82, 7, 0x80, 5, 1, 2, 3, 4, 5][..], &*to_vec(&blah).unwrap());
    assert_eq!(&[0x83, 7, 0x80, 5, 1, 2, 3, 4, 5][..], &*to_vec(&lah).unwrap());
}


/*
   #[test]
   fn integer() {
   let small_integer = Integer::from(5);
   let multi_byte_integer = Integer::from(0xffff);
   assert_eq!(&[2, 1, 5][..], &*to_vec(&small_integer).unwrap());
   assert_eq!(&[2, 3, 0, 0xff, 0xff][..], &*to_vec(&multi_byte_integer).unwrap());
   }

#[test]
fn object_identifier() {
use core::types::ObjectIdentifier;

let just_root: Vec<u8> = to_vec(&ObjectIdentifier::new(vec![1, 2]).unwrap()).unwrap();
let itu: Vec<u8> = to_vec(&ObjectIdentifier::new(vec![2, 999, 3]).unwrap()).unwrap();
let rsa: Vec<u8> =
to_vec(&ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap()).unwrap();

assert_eq!(&[0x6, 0x1, 0x2a][..], &*just_root);
assert_eq!(&[0x6, 0x3, 0x88, 0x37, 0x03][..], &*itu);
assert_eq!(&[0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..], &*rsa);
}

#[test]
fn bit_string() {
use core::types::BitString;

let bitvec = BitString::from_bytes(&[0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0]);

assert_eq!(
&[0x3u8, 0x7, 0x04, 0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0][..],
&*to_vec(&bitvec).unwrap()
);
}
use typenum::consts::*;

#[test]
fn implicit_prefix() {
use typenum::consts::*;
use core::identifier::constant::*;
type MyInteger = core::types::Implicit<Universal, U7, u64>;

let new_int = MyInteger::new(5);

assert_eq!(&[7, 1, 5], &*to_vec(&new_int).unwrap());
}

#[test]
fn explicit_prefix() {
use typenum::consts::*;
use core::identifier::constant::*;
type MyInteger = core::types::Explicit<Context, U0, u64>;

let new_int = MyInteger::new(5);

assert_eq!(&[0xA0, 3, 2, 1, 5], &*to_vec(&new_int).unwrap());
}
*/

