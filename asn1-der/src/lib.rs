mod decoder;
mod encoder;
mod tag;
mod value;

pub use decoder::from_der;
pub use encoder::to_der;
pub use value::Value;

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use decoder::parse_value;
    use core::types::ObjectIdentifier;

    #[test]
    fn bool() {
        assert_eq!(true, from_der(&to_der(true)).unwrap());
        assert_eq!(false, from_der(&to_der(false)).unwrap());
    }

    #[test]
    fn object_identifier() {
        let iso = ObjectIdentifier::new(vec![1, 2]).unwrap();
        let us_ansi = ObjectIdentifier::new(vec![1, 2, 840]).unwrap();
        let rsa = ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap();
        let pkcs = ObjectIdentifier::new(vec![1, 2, 840, 113549, 1]).unwrap();

        assert_eq!(iso.clone(), from_der(&to_der(iso)).unwrap());
        assert_eq!(us_ansi.clone(), from_der(&to_der(us_ansi)).unwrap());
        assert_eq!(rsa.clone(), from_der(&to_der(rsa)).unwrap());
        assert_eq!(pkcs.clone(), from_der(&to_der(pkcs)).unwrap());
    }

    macro_rules! integer_tests {
        ($($name:ident : $integer:ty),*) => {
            $(
                #[test]
                fn $name () {
                    let min = <$integer>::min_value();
                    let max = <$integer>::max_value();

                    assert_eq!(min, from_der(&to_der(min)).unwrap());
                    assert_eq!(max, from_der(&to_der(max)).unwrap());
                }
            )*
        }
    }

    integer_tests! {
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        u128: u128
    }
}
