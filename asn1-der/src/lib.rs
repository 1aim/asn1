mod decoder;
mod encoder;
mod tag;
mod value;
mod error;

pub use decoder::from_slice;
pub use encoder::to_vec;
pub use tag::Tag;
pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};

    #[test]
    fn bool() {
        assert_eq!(true, from_slice(&to_vec(&true).unwrap()).unwrap());
        assert_eq!(false, from_slice(&to_vec(&false).unwrap()).unwrap());
    }

    /*
    #[test]
    fn octet_string() {
        let a = vec![1u8, 2, 3, 4, 5];
        let b = vec![5u8, 4, 3, 2, 1];

        assert_eq!(a.clone(), from_slice::<Vec<u8>>(&to_vec(&a).expect("encoding")).expect("decoding"));
        assert_eq!(b.clone(), from_slice::<Vec<u8>>(&to_vec(&b).unwrap()).unwrap());
    }
    */

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
            0x30,    // Sequence tag
            9,       // Length
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

        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Kind {
            Number(u8),
            Vec(Vec<u8>),
        }

        assert_eq!(Foo::Ein, from_slice(&to_vec(&Foo::Ein).unwrap()).unwrap());
        assert_eq!(Foo::Zwei, from_slice(&to_vec(&Foo::Zwei).unwrap()).unwrap());
        assert_eq!(Foo::Drei, from_slice(&to_vec(&Foo::Drei).unwrap()).unwrap());

        assert_eq!(Kind::Number(5), from_slice(&to_vec(&Kind::Number(5)).unwrap()).unwrap());
        assert_eq!(Kind::Vec(vec![1, 2, 3, 4]), from_slice(&to_vec(&Kind::Vec(vec![1, 2, 3, 4])).unwrap()).unwrap());
    }

    /*
    #[test]
    fn object_identifier() {
        let iso = ObjectIdentifier::new(vec![1, 2]).unwrap();
        let us_ansi = ObjectIdentifier::new(vec![1, 2, 840]).unwrap();
        let rsa = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();
        let pkcs = ObjectIdentifier::new(vec![1, 2, 840, 113549, 1]).unwrap();

        assert_eq!(iso.clone(), from_slice(&to_vec(iso)).unwrap());
        assert_eq!(us_ansi.clone(), from_slice(&to_vec(us_ansi)).unwrap());
        assert_eq!(rsa.clone(), from_slice(&to_vec(rsa)).unwrap());
        assert_eq!(pkcs.clone(), from_slice(&to_vec(pkcs)).unwrap());
    }
    */
}
