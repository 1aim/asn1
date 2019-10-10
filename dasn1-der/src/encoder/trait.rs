use super::Serializer;

pub trait DerEncodable {
    fn encode(&self) -> Vec<u8>;
}

impl DerEncodable for bool {
    fn encode(&self) -> Vec<u8> {
        Serializer::serialize_to_vec(self, false).unwrap().output
    }
}

impl<T: DerEncodable> DerEncodable for Option<T> {
    fn encode(&self) -> Vec<u8> {
        match self {
            Some(v) => v.encode(),
            None => Vec::new(),
        }
    }
}

macro_rules! arrays {
    ($($num:tt)+) => {

        $(
            impl<T: DerEncodable> DerEncodable for [T; $num] {
                fn encode(&self) -> Vec<u8> {
                    Serializer::serialize_to_vec(self, false).unwrap().output

                }
            }
        )+
    }
}

macro_rules! integers {
    ($($int:ty)+) => {
        $(
            impl DerEncodable for $int {
                fn encode(&self) -> Vec<u8> {
                    Serializer::serialize_to_vec(self, false).unwrap().output
                }
            }
        )+
    }
}

macro_rules! tuples {
    ($($tuple:ty)+) => {
        $(
            impl<T: DerEncodable> DerEncodable for $tuple {
                fn encode(&self) -> Vec<u8> {
                    Serializer::serialize_to_vec(self, false).unwrap().output
                }
            }
        )+
    }
}

arrays! {
    1 2 3 4 5 6 7 8 9 10
    11 12 13 14 15 16 17 18 19 20
}

integers! {
    u8 u16 u32 u64 u128 i8 i16 i32 i64 i128
}

tuples! {
    (T, T)
    (T, T, T)
    (T, T, T, T)
}
