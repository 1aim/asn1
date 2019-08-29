pub mod number;
pub mod buffer;

pub use self::buffer::Buffer;

pub trait PerEncodable {
    fn encode(&self) -> bit_vec::BitVec;
}
