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
                    ser::number::encode_constrained_whole_number(*self, range)
                }
            }
        )+
    }
}

integers!(u8 u16 u32 u64 u128);
