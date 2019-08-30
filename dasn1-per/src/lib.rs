use std::ops::RangeBounds;

pub mod ser;

pub use ser::Buffer;

pub fn encode<T: PerEncodable>(value: &T) -> Vec<u8> {
    value.encode().to_bytes()
}

pub trait PerEncodable {
    fn encode(&self) -> Buffer;
}

impl PerEncodable for bool {
    fn encode(&self) -> Buffer {
        Buffer::from_elem(1, *self)
    }
}

impl<T: PerEncodable> PerEncodable for Option<T> {
    fn encode(&self) -> Buffer {
        // Encode if available else provide an empty buffer.
        self.as_ref().map(T::encode).unwrap_or_default()
    }
}

impl<T: PerEncodable> PerEncodable for (T, T) {
    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.push_field_list(self.0.encode());
        buffer.push_field_list(self.1.encode());
        buffer
    }
}

macro_rules! integers {
    ($($int:ty)+) => {
        $(
            impl PerEncodable for $int {
                fn encode(&self) -> Buffer {
                    let range = <$int>::min_value()..=<$int>::max_value();
                    self.encode_with_constraint(range)
                }
            }

            impl ConstrainedValue for $int {
                type RangeBound = $int;
                fn encode_with_constraint<R: RangeBounds<Self::RangeBound>>(&self, range: R) -> Buffer {
                    ser::number::encode_integer(*self, range)
                }
            }
        )+
    }
}

integers!(u8 u16 u32 u64 u128);

pub trait ConstrainedValue: PerEncodable {
    type RangeBound;

    fn encode_with_constraint<R: RangeBounds<Self::RangeBound>>(&self, range: R) -> Buffer;
}

impl<T: ConstrainedValue> ConstrainedValue for Option<T> {
    type RangeBound = T::RangeBound;

    fn encode_with_constraint<R: RangeBounds<Self::RangeBound>>(&self, range: R) -> Buffer {
        match self {
            Some(val) => val.encode_with_constraint(range),
            None => Buffer::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_encode_to_correct_width() {
        assert_eq!(8, 0u8.encode().len());
        assert_eq!(16, 0u16.encode().len());
        assert_eq!(32, 0u32.encode().len());
        assert_eq!(64, 0u64.encode().len());
        assert_eq!(128, 0u128.encode().len());
    }

    #[test]
    fn push_to_field_list() {
        let mut a = 1u8.encode();
        let mut b = 2u16.encode();
        let mut c = dbg!(3u8.encode());

        a.push_field_list(b);
        a.push_field_list(c);
        dbg!(&a);



        assert_eq!(&[01, 00, 02, 03][..], &*a.to_bytes());
    }
}
