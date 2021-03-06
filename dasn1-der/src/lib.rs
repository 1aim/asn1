//! # ASN.1 Distingushed Encoding Rules (DER)
//! Distingushed Encoding Rules are a superset of the ASN.1 Basic Encoding Rules
//! (BER), designed to be canonical (An encoding where there is only a single
//! way to encode a value) and for when you have all of the data needed to
//! encode. DER is used in frequently in cryptography (X.509 certificates,
//! PKCS#12).
#![deny(missing_docs)]

mod decoder;
mod encoder;
pub mod identifier;
pub mod error;

pub use decoder::from_slice;
pub use encoder::to_vec;
pub use error::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use core::types::*;
    use serde_derive::{Deserialize, Serialize};
    use typenum::consts::*;
    use core::identifier::constant::*;

    #[test]
    fn bool() {
        assert_eq!(true, from_slice(&to_vec(&true).unwrap()).unwrap());
        assert_eq!(false, from_slice(&to_vec(&false).unwrap()).unwrap());
    }

    #[test]
    fn octet_string() {
        let a = OctetString::from(vec![1u8, 2, 3, 4, 5]);
        let b = OctetString::from(vec![5u8, 4, 3, 2, 1]);

        assert_eq!(
            a,
            from_slice(&to_vec(&a).expect("encoding")).expect("decoding")
        );
        assert_eq!(b, from_slice(&to_vec(&b).unwrap()).unwrap());
    }

    #[test]
    fn universal_string() {
        let name = "Jones";
        assert_eq!(
            name,
            from_slice::<String>(&*to_vec(&name).unwrap()).unwrap()
        );
    }

    macro_rules! integer_tests {
        ($($integer:ident)*) => {
            $(
                #[test]
                fn $integer() {
                    let min = <$integer>::min_value();
                    let max = <$integer>::max_value();

                    assert_eq!(min, from_slice(&to_vec(&min).unwrap()).unwrap());
                    assert_eq!(max, from_slice(&to_vec(&max).unwrap()).unwrap());
                }
            )*
        }
    }

    integer_tests!(i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);

    #[test]
    fn sequence() {
        #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
        struct Bools {
            a: bool,
            b: bool,
            c: bool,
        }

        let raw = &[
            0x30, // Sequence tag
            9,    // Length
            1, 1, 0xff, // A
            1, 1, 0, // B
            1, 1, 0xff, // C
        ][..];

        let default = Bools {
            a: true,
            b: false,
            c: true,
        };
        assert_eq!(default, from_slice(&raw).unwrap());
        assert_eq!(raw, &*to_vec(&default).unwrap());

        // The representation of SEQUENCE and SEQUENCE OF are the same in this case.
        let bools_vec = vec![true, false, true];

        assert_eq!(bools_vec, from_slice::<Vec<bool>>(&raw).unwrap());
        assert_eq!(raw, &*to_vec(&bools_vec).unwrap());
    }

    #[test]
    fn choice() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Foo {
            Ein,
            Zwei,
            Drei,
        }

        impl Enumerable for Foo {}

        let ein = Enumerated::new(Foo::Ein);
        let zwei = Enumerated::new(Foo::Zwei);
        let drei = Enumerated::new(Foo::Drei);

        assert_eq!(ein, from_slice(&to_vec(&ein).unwrap()).unwrap());
        assert_eq!(zwei, from_slice(&to_vec(&zwei).unwrap()).unwrap());
        assert_eq!(drei, from_slice(&to_vec(&drei).unwrap()).unwrap());
    }

    #[test]
    fn choice_newtype_variant() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Foo {
            Bar(Implicit<Context, U0, bool>),
            Baz(Implicit<Context, U1, OctetString>),
        }

        let bar = Foo::Bar(Implicit::new(true));
        let baz = Foo::Baz(Implicit::new(OctetString::from(vec![1, 2, 3, 4, 5])));

        assert_eq!(bar, from_slice(&to_vec(&bar).unwrap()).unwrap());
        assert_eq!(baz, from_slice(&to_vec(&baz).unwrap()).unwrap());
    }

    #[test]
    fn sequence_in_sequence_in_choice() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum FooExtern {
            Bar(Implicit<Context, U0, BarData>),
        }

        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct BarData {
            data: OctetString,
        }

        let bar_extern = FooExtern::Bar(Implicit::new(BarData {
            data: OctetString::from(vec![1, 2, 3, 4]),
        }));
        let extern_encoded = to_vec(&bar_extern).unwrap();

        assert_eq!(bar_extern, from_slice(&extern_encoded).unwrap());
    }

    #[test]
    fn response() {
        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
        struct Response {
            status: Status,
            body: Body,
        }

        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
        enum Status {
            Success(Implicit<Context, U0, ()>),
            Error(Implicit<Context, U1, u8>),
        }

        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
        struct Body {
            data: OctetString,
        }

        let response = Response {
            status: Status::Success(Implicit::new(())),
            body: Body {
                data: OctetString::from(vec![1, 2, 3, 4, 5]),
            },
        };

        assert_eq!(response, from_slice(&to_vec(&response).unwrap()).unwrap());
    }

    /*
    #[test]
    fn optional() {
        env_logger::init();
        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
        struct Struct {
            a: Optional<u8>,
        }

        let none = Struct { a: None.into() };
        let raw = to_vec(&none).unwrap();
        assert_eq!(&[0x30, 0][..], &*raw);
        assert_eq!(none, from_slice(&raw).unwrap());

        let some = Struct { a: Some(100).into() };
        assert_eq!(some, from_slice(&to_vec(&some).unwrap()).unwrap());
    }

    #[test]
    fn sequence_with_option() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            a: u8,
            b: Optional<u8>,
        }

        let some = Foo { a: 1, b: Some(2).into() };
        let none = Foo { a: 1, b: None.into() };

        assert_eq!(some, from_slice(&to_vec(&some).unwrap()).unwrap());
        assert_eq!(none, from_slice(&to_vec(&none).unwrap()).unwrap());
    }
    */


    #[test]
    fn object_identifier() {
        use core::types::ObjectIdentifier;

        let iso = ObjectIdentifier::new(vec![1, 2]).unwrap();
        let us_ansi = ObjectIdentifier::new(vec![1, 2, 840]).unwrap();
        let rsa = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();
        let pkcs = ObjectIdentifier::new(vec![1, 2, 840, 113549, 1]).unwrap();

        assert_eq!(iso.clone(), from_slice(&to_vec(&iso).unwrap()).unwrap());
        assert_eq!(
            us_ansi.clone(),
            from_slice(&to_vec(&us_ansi).unwrap()).unwrap()
        );
        assert_eq!(rsa.clone(), from_slice(&to_vec(&rsa).unwrap()).unwrap());
        assert_eq!(pkcs.clone(), from_slice(&to_vec(&pkcs).unwrap()).unwrap());
    }

    #[test]
    fn long_sequence() {
        let vec = vec![5u8; 0xffff];
        assert_eq!(vec, from_slice::<Vec<u8>>(&to_vec(&vec).unwrap()).unwrap());
    }

    #[test]
    fn bit_string() {
        use core::types::BitString;

        let bits = BitString::from_bytes(&[0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0]);

        assert_eq!(bits, from_slice(&to_vec(&bits).unwrap()).unwrap());
    }

    #[test]
    fn implicit_prefix() {
        type MyInteger = core::types::Implicit<Context, U0, u64>;

        let new_int = MyInteger::new(5);

        assert_eq!(new_int, from_slice(&to_vec(&new_int).unwrap()).unwrap());
    }

    #[test]
    fn explicit_prefix() {
        type MyInteger = core::types::Explicit<Context, U0, u64>;

        let new_int = MyInteger::new(5);

        assert_eq!(new_int, from_slice(&to_vec(&new_int).unwrap()).unwrap());
    }

    #[test]
    fn nested_enum() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        enum Alpha {
            A(Implicit<Context, U0, Charlie>),
            B(Implicit<Context, U1, Charlie>),
        }


        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        enum Bravo {
            A,
            B,
        }

        impl Enumerable for Bravo {}

        type Charlie = Enumerated<Bravo>;

        let input = Alpha::A(Implicit::new(Enumerated::new(Bravo::B)));

        assert_eq!(input, from_slice(&to_vec(&input).unwrap()).unwrap())
    }
}
