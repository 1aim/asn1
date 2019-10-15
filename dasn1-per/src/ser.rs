pub mod buffer;
pub mod number;

pub use self::buffer::Buffer;

pub trait PerEncodable {
    fn encode(&self) -> bit_vec::BitVec;
}
