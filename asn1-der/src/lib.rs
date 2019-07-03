mod decoder;
mod encoder;
mod error;

pub use decoder::from_slice;
pub use encoder::to_vec;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use core::types::OctetString;

    #[test]
    fn bool() {
        assert_eq!(true, from_slice(&to_vec(&true).unwrap()).unwrap());
        assert_eq!(false, from_slice(&to_vec(&false).unwrap()).unwrap());
    }

    #[test]
    fn octet_string() {
        use core::types::OctetString;
        let a = OctetString::from(vec![1u8, 2, 3, 4, 5]);
        let b = OctetString::from(vec![5u8, 4, 3, 2, 1]);

        assert_eq!(a, from_slice(&to_vec(&a).expect("encoding")).expect("decoding"));
        assert_eq!(b, from_slice(&to_vec(&b).unwrap()).unwrap());
    }

    #[test]
    fn universal_string() {
        let name = "Jones";
        assert_eq!(name, from_slice::<String>(&*to_vec(&name).unwrap()).unwrap());
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

        assert_eq!(Foo::Ein, from_slice(&to_vec(&Foo::Ein).unwrap()).unwrap());
        assert_eq!(Foo::Zwei, from_slice(&to_vec(&Foo::Zwei).unwrap()).unwrap());
        assert_eq!(Foo::Drei, from_slice(&to_vec(&Foo::Drei).unwrap()).unwrap());
    }

    #[test]
    fn choice_newtype_variant() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum Foo {
            Bar(bool),
            Baz(OctetString),
        }

        let bar = Foo::Bar(true);
        let baz = Foo::Baz(OctetString::from(vec![1, 2, 3, 4, 5]));

        assert_eq!(bar, from_slice(&to_vec(&bar).unwrap()).unwrap());
        assert_eq!(baz, from_slice(&to_vec(&baz).unwrap()).unwrap());
    }

    #[test]
    fn sequence_in_sequence_in_choice() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum FooInline {
            Bar {
                data: OctetString,
            }
        }

        // FooExtern should have the same encoding as FooInline.
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        enum FooExtern {
            Bar(BarData)
        }

        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct BarData {
            data: OctetString,
        }

        let bar = FooInline::Bar { data: OctetString::from(vec![1, 2, 3, 4])};
        let bar_extern = FooExtern::Bar(BarData { data: OctetString::from(vec![1, 2, 3, 4])});
        let inline_encoded = to_vec(&bar).unwrap();
        let extern_encoded = to_vec(&bar_extern).unwrap();

        assert_eq!(bar, from_slice(&inline_encoded).unwrap());
        assert_eq!(bar_extern, from_slice(&inline_encoded).unwrap());

        assert_eq!(bar, from_slice(&extern_encoded).unwrap());
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
            Success,
            Error(u8),
        }

        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
        struct Body {
            data: OctetString,
        }

        let response = Response {
            status: Status::Success,
            body: Body {
                data: OctetString::from(vec![1, 2, 3, 4, 5])
            }
        };

        assert_eq!(response, from_slice(&to_vec(&response).unwrap()).unwrap());
    }

    #[test]
    fn string() {
        let string = "Hello World!";

        assert_eq!(string, from_slice::<String>(&to_vec(&string).unwrap()).unwrap());
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
